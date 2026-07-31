package com.fumbbl.ffb.factory;

import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.Weather;
import com.fumbbl.ffb.model.Game;
import com.fumbbl.ffb.model.Player;
import com.fumbbl.ffb.model.RosterPlayer;
import com.fumbbl.ffb.modifiers.ModifierType;
import com.fumbbl.ffb.modifiers.PickupContext;
import com.fumbbl.ffb.modifiers.PickupModifier;
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
 * Mirror of ffb-rust crates/ffb-mechanics/src/modifiers/pickup_modifier_factory.rs tests.
 */
public class PickupModifierFactoryTest {

	private GameState gameState;
	private Game game;
	private PickupModifierFactory factory;

	@BeforeEach
	void setUp() {
		init(GameFixture.createGameState(3));
	}

	private void init(GameState gs) {
		gameState = gs;
		game = gameState.getGame();
		factory = new PickupModifierFactory();
		factory.initialize(game);
		GameFixture.placePlayer(gameState, "home1", 5, 5);
	}

	private Player<?> player() {
		return game.getPlayerById("home1");
	}

	private PickupContext context() {
		return new PickupContext(game, player());
	}

	private void addSkill(String skillName) {
		((RosterPlayer) player()).addSkill(GameFixture.skill(game, skillName));
	}

	private Set<String> names(Set<PickupModifier> modifiers) {
		return modifiers.stream().map(PickupModifier::getName).collect(Collectors.toSet());
	}

	// rust: find_registered_modifiers_includes_extra_arms_and_big_hand
	@Test
	public void findRegisteredModifiersIncludesExtraArmsAndBigHand() {
		Set<PickupModifier> mods = game.getModifierAggregator().getPickupModifiers();
		Set<String> names = names(mods);
		assertTrue(names.contains("Extra Arms"));
		assertTrue(names.contains("Big Hand"));
		assertEquals(2, mods.size());
	}

	// rust: nice_weather_no_modifiers
	@Test
	public void niceWeatherNoModifiers() {
		assertTrue(factory.findModifiers(context()).isEmpty(),
			"Nice weather should yield no pickup modifiers");
	}

	// rust: pouring_rain_adds_one_modifier
	@Test
	public void pouringRainAddsOneModifier() {
		game.getFieldModel().setWeather(Weather.POURING_RAIN);
		Set<PickupModifier> mods = factory.findModifiers(context());
		assertEquals(1, mods.size(), "Pouring Rain should add one pickup modifier");
		assertEquals(1, mods.iterator().next().getModifier());
	}

	// rust: minimum_roll_no_modifiers
	@Test
	public void minimumRollNoModifiers() {
		assertEquals(3, new com.fumbbl.ffb.mechanics.bb2025.AgilityMechanic()
			.minimumRollPickup(player(), Collections.emptySet()));
	}

	// rust: minimum_roll_with_rain_modifier
	@Test
	public void minimumRollWithRainModifier() {
		PickupModifier m = new PickupModifier("Pouring Rain", 1, ModifierType.REGULAR);
		assertEquals(4, new com.fumbbl.ffb.mechanics.bb2025.AgilityMechanic()
			.minimumRollPickup(player(), Collections.singleton(m)));
	}

	// rust: minimum_roll_never_below_two
	@Test
	public void minimumRollNeverBelowTwo() {
		((RosterPlayer) player()).setAgility(1);
		assertEquals(2, new com.fumbbl.ffb.mechanics.bb2025.AgilityMechanic()
			.minimumRollPickup(player(), Collections.emptySet()));
	}

	// rust: for_name_returns_rain_modifier
	@Test
	public void forNameReturnsRainModifier() {
		assertNotNull(factory.forName("Pouring Rain"));
		assertNull(factory.forName("NonExistent"));
	}

	// rust: find_skill_modifiers_extra_arms_applies
	@Test
	public void findSkillModifiersExtraArmsApplies() {
		addSkill("Extra Arms");
		Set<PickupModifier> mods = factory.findModifiers(context());
		PickupModifier extraArms = mods.stream().filter(m -> "Extra Arms".equals(m.getName()))
			.findFirst().orElse(null);
		assertNotNull(extraArms, "Extra Arms should apply");
		assertEquals(-1, extraArms.getModifier());
	}

	// rust: find_skill_modifiers_big_hand_marker_edition_wording
	@Test
	public void findSkillModifiersBigHandMarkerEditionWording() {
		addSkill("Big Hand");
		Set<PickupModifier> mods = factory.findModifiers(context());
		PickupModifier bigHand = mods.stream().filter(m -> "Big Hand".equals(m.getName()))
			.findFirst().orElse(null);
		assertNotNull(bigHand, "Big Hand marker should be present");
		assertEquals(0, bigHand.getModifier());
		assertEquals("0 ignoring all negative modifiers due to Big Hand", bigHand.getReportString());
		assertTrue(bigHand.isModifierIncluded());

		init(GameFixture.createGameState(3, RulesCollection.Rules.BB2016));
		addSkill("Big Hand");
		mods = factory.findModifiers(context());
		bigHand = mods.stream().filter(m -> "Big Hand".equals(m.getName()))
			.findFirst().orElse(null);
		assertNotNull(bigHand, "mixed Big Hand marker should be present in BB2016");
		assertEquals("0 ignoring all tackle zones and weather effects due to Big Hand",
			bigHand.getReportString());
	}
}
