package com.fumbbl.ffb.factory;

import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.Weather;
import com.fumbbl.ffb.factory.common.GoForItModifierFactory;
import com.fumbbl.ffb.model.Game;
import com.fumbbl.ffb.model.Player;
import com.fumbbl.ffb.model.RosterPlayer;
import com.fumbbl.ffb.modifiers.GoForItContext;
import com.fumbbl.ffb.modifiers.GoForItModifier;
import com.fumbbl.ffb.server.DiceInterpreter;
import com.fumbbl.ffb.server.GameState;
import com.fumbbl.ffb.server.fixture.GameFixture;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import java.util.Collections;
import java.util.Set;
import java.util.stream.Collectors;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNotNull;
import static org.junit.jupiter.api.Assertions.assertNull;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirror of ffb-rust crates/ffb-mechanics/src/modifiers/go_for_it_modifier_factory.rs tests.
 */
public class GoForItModifierFactoryTest {

	private GameState gameState;
	private Game game;
	private GoForItModifierFactory factory;

	@BeforeEach
	void setUp() {
		init(GameFixture.createGameState(3));
	}

	private void init(GameState gs) {
		gameState = gs;
		game = gameState.getGame();
		factory = new GoForItModifierFactory();
		factory.initialize(game);
		GameFixture.placePlayer(gameState, "home1", 5, 5);
	}

	private Player<?> player() {
		return game.getPlayerById("home1");
	}

	private GoForItContext context() {
		return new GoForItContext(game, player(), Collections.emptySet());
	}

	private Set<String> names(Set<GoForItModifier> modifiers) {
		return modifiers.stream().map(GoForItModifier::getName).collect(Collectors.toSet());
	}

	// rust: minimum_roll_base_is_two
	@Test
	public void minimumRollBaseIsTwo() {
		assertEquals(2, DiceInterpreter.getInstance().minimumRollGoingForIt(Collections.emptySet()));
	}

	// rust: minimum_roll_adds_modifier_total
	@Test
	public void minimumRollAddsModifierTotal() {
		GoForItModifier m = new GoForItModifier("Blizzard", 1);
		assertEquals(3, DiceInterpreter.getInstance().minimumRollGoingForIt(Collections.singleton(m)));
	}

	// rust: minimum_roll_never_below_two
	@Test
	public void minimumRollNeverBelowTwo() {
		GoForItModifier m = new GoForItModifier("Bonus", -5);
		assertEquals(2, DiceInterpreter.getInstance().minimumRollGoingForIt(Collections.singleton(m)));
	}

	// rust: nice_weather_no_modifiers
	@Test
	public void niceWeatherNoModifiers() {
		assertTrue(factory.findModifiers(context()).isEmpty(),
			"Nice weather should yield no GFI modifiers");
	}

	// rust: blizzard_weather_adds_one_modifier
	@Test
	public void blizzardWeatherAddsOneModifier() {
		game.getFieldModel().setWeather(Weather.BLIZZARD);
		Set<GoForItModifier> mods = factory.findModifiers(context());
		assertEquals(1, mods.size(), "Blizzard should add one GFI modifier");
		assertEquals(3, DiceInterpreter.getInstance().minimumRollGoingForIt(mods));
	}

	// rust: for_name_returns_blizzard_modifier
	@Test
	public void forNameReturnsBlizzardModifier() {
		assertNotNull(factory.forName("Blizzard"));
		assertNull(factory.forName("NonExistent"));
	}

	// rust: find_skill_modifiers_drunkard_applies_in_bb2025
	// (mixed.Drunkard registers an unconditional GoForItModifier("Drunkard", 1) for
	// BB2020/BB2025; the Rust engine previously had no GFI skill-modifier path at all,
	// so Drunkard players rushed at 2+ instead of 3+ — fixed against this test.)
	@Test
	public void findSkillModifiersDrunkardAppliesInBb2025() {
		((RosterPlayer) player()).addSkill(GameFixture.skill(game, "Drunkard"));
		Set<GoForItModifier> mods = factory.findModifiers(context());
		GoForItModifier drunkard = mods.stream().filter(m -> "Drunkard".equals(m.getName()))
			.findFirst().orElse(null);
		assertNotNull(drunkard, "Drunkard modifier should apply");
		assertEquals(1, drunkard.getModifier());
		assertEquals(3, DiceInterpreter.getInstance().minimumRollGoingForIt(mods));
	}

	// rust: find_registered_modifiers_drunkard_bb2025_not_bb2016
	@Test
	public void findRegisteredModifiersDrunkardBb2025NotBb2016() {
		Set<GoForItModifier> mods = game.getModifierAggregator().getGoForItModifiers();
		assertEquals(1, mods.size());
		assertEquals("Drunkard", mods.iterator().next().getName());

		init(GameFixture.createGameState(3, RulesCollection.Rules.BB2016));
		assertTrue(game.getModifierAggregator().getGoForItModifiers().isEmpty());
	}
}
