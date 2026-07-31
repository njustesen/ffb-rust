package com.fumbbl.ffb.factory;

import com.fumbbl.ffb.CatchScatterThrowInMode;
import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.Weather;
import com.fumbbl.ffb.model.Game;
import com.fumbbl.ffb.model.Player;
import com.fumbbl.ffb.model.RosterPlayer;
import com.fumbbl.ffb.modifiers.CatchContext;
import com.fumbbl.ffb.modifiers.CatchModifier;
import com.fumbbl.ffb.modifiers.ModifierType;
import com.fumbbl.ffb.server.GameState;
import com.fumbbl.ffb.server.fixture.GameFixture;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import java.util.Collections;
import java.util.List;
import java.util.Set;
import java.util.stream.Collectors;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertNotNull;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirror of ffb-rust crates/ffb-mechanics/src/modifiers/catch_modifier_factory.rs tests.
 * The factory is initialized against a GameFixture game (players start in the reserve box,
 * so only explicitly placed players project tackle zones).
 */
public class CatchModifierFactoryTest {

	private GameState gameState;
	private Game game;
	private CatchModifierFactory factory;

	@BeforeEach
	void setUp() {
		init(GameFixture.createGameState(3));
	}

	private void init(GameState gs) {
		gameState = gs;
		game = gameState.getGame();
		factory = new CatchModifierFactory();
		factory.initialize(game);
		GameFixture.placePlayer(gameState, "home1", 5, 5);
	}

	private Player<?> catcher() {
		return game.getPlayerById("home1");
	}

	private CatchContext context(CatchScatterThrowInMode mode) {
		return new CatchContext(game, catcher(), mode, null);
	}

	private void addSkill(String playerId, String skillName) {
		((RosterPlayer) game.getPlayerById(playerId)).addSkill(GameFixture.skill(game, skillName));
	}

	private Set<String> names(Set<CatchModifier> modifiers) {
		return modifiers.stream().map(CatchModifier::getName).collect(Collectors.toSet());
	}

	// rust: nice_weather_no_modifiers
	@Test
	public void niceWeatherNoModifiers() {
		Set<CatchModifier> mods = factory.findModifiers(context(CatchScatterThrowInMode.CATCH_SCATTER));
		assertFalse(names(mods).contains("Pouring Rain"),
			"Pouring Rain modifier should be absent in NICE weather");
	}

	// rust: pouring_rain_adds_modifier
	@Test
	public void pouringRainAddsModifier() {
		game.getFieldModel().setWeather(Weather.POURING_RAIN);
		Set<CatchModifier> mods = factory.findModifiers(context(CatchScatterThrowInMode.CATCH_HAND_OFF));
		assertTrue(names(mods).contains("Pouring Rain"), "Pouring rain modifier should be present");
	}

	// rust: find_applicable_selects_single_tacklezone_modifier_by_count
	@Test
	public void findModifiersSelectsSingleTacklezoneModifierByCount() {
		GameFixture.placePlayer(gameState, "away1", 6, 5);
		Set<CatchModifier> mods = factory.findModifiers(context(CatchScatterThrowInMode.CATCH_HAND_OFF));
		List<CatchModifier> tzMods = mods.stream()
			.filter(m -> m.getType() == ModifierType.TACKLEZONE).collect(Collectors.toList());
		assertEquals(1, tzMods.size(),
			"exactly one tacklezone modifier should be selected, got: " + names(mods));
		assertEquals("1 Tacklezone", tzMods.get(0).getName());
		assertFalse(mods.stream().anyMatch(m -> m.getType() == ModifierType.DISTURBING_PRESENCE),
			"no disturbing-presence modifier without DP opponents");
	}

	// rust: minimum_roll_catch_base
	@Test
	public void minimumRollCatchBase() {
		assertEquals(3, new com.fumbbl.ffb.mechanics.bb2025.AgilityMechanic()
			.minimumRollCatch(catcher(), Collections.emptySet()));
	}

	// rust: minimum_roll_catch_never_below_two
	@Test
	public void minimumRollCatchNeverBelowTwo() {
		((RosterPlayer) catcher()).setAgility(1);
		CatchModifier bonus = new CatchModifier("Bonus", -5, ModifierType.REGULAR);
		assertEquals(2, new com.fumbbl.ffb.mechanics.bb2025.AgilityMechanic()
			.minimumRollCatch(catcher(), Collections.singleton(bonus)));
	}

	// rust: minimum_roll_catch_with_positive_modifier_raises_minimum
	@Test
	public void minimumRollCatchWithPositiveModifierRaisesMinimum() {
		CatchModifier gloves = new CatchModifier("Gloves", 1, ModifierType.REGULAR);
		assertEquals(4, new com.fumbbl.ffb.mechanics.bb2025.AgilityMechanic()
			.minimumRollCatch(catcher(), Collections.singleton(gloves)));
	}

	// rust: find_registered_modifiers_includes_extra_arms_diving_catch_and_nerves_of_steel
	@Test
	public void findRegisteredModifiersIncludesExtraArmsDivingCatchAndNervesOfSteel() {
		Set<CatchModifier> mods = game.getModifierAggregator().getCatchModifiers();
		Set<String> names = names(mods);
		assertTrue(names.contains("Extra Arms"));
		assertTrue(names.contains("Diving Catch"));
		assertTrue(names.contains("Nerves of Steel"));
		assertEquals(3, mods.size());
	}

	// rust: find_skill_modifiers_extra_arms_always_applies
	@Test
	public void findSkillModifiersExtraArmsAlwaysApplies() {
		addSkill("home1", "Extra Arms");
		Set<CatchModifier> mods = factory.findModifiers(context(CatchScatterThrowInMode.CATCH_HAND_OFF));
		CatchModifier extraArms = mods.stream().filter(m -> "Extra Arms".equals(m.getName()))
			.findFirst().orElse(null);
		assertNotNull(extraArms, "Extra Arms should apply");
		assertEquals(-1, extraArms.getModifier());
	}

	// rust: find_skill_modifiers_diving_catch_on_accurate_pass
	@Test
	public void findSkillModifiersDivingCatchOnAccuratePass() {
		addSkill("home1", "Diving Catch");
		Set<CatchModifier> mods = factory.findModifiers(context(CatchScatterThrowInMode.CATCH_ACCURATE_PASS));
		assertTrue(names(mods).contains("Diving Catch"), "Diving Catch should apply on accurate pass");
	}

	// rust: find_skill_modifiers_diving_catch_not_on_inaccurate
	@Test
	public void findSkillModifiersDivingCatchNotOnInaccurate() {
		addSkill("home1", "Diving Catch");
		Set<CatchModifier> mods = factory.findModifiers(context(CatchScatterThrowInMode.CATCH_SCATTER));
		assertFalse(names(mods).contains("Diving Catch"), "Diving Catch should NOT apply on scatter");
	}

	// rust: find_skill_modifiers_nerves_of_steel_marker
	@Test
	public void findSkillModifiersNervesOfSteelMarker() {
		addSkill("home1", "Nerves of Steel");
		Set<CatchModifier> mods = factory.findModifiers(context(CatchScatterThrowInMode.CATCH_HAND_OFF));
		CatchModifier nos = mods.stream().filter(m -> "Nerves of Steel".equals(m.getName()))
			.findFirst().orElse(null);
		assertNotNull(nos, "Nerves of Steel marker should be present");
		assertEquals(0, nos.getModifier());
		assertEquals("0 for tackle zones due to Nerves of Steel", nos.getReportString());
		assertTrue(nos.isModifierIncluded());
	}

	// rust: for_rules_bb2016_includes_accurate_pass_and_hand_off
	@Test
	public void forRulesBb2016IncludesAccuratePassAndHandOff() {
		init(GameFixture.createGameState(3, RulesCollection.Rules.BB2016));
		Set<CatchModifier> mods = factory.findModifiers(context(CatchScatterThrowInMode.CATCH_HAND_OFF));
		assertTrue(names(mods).contains("Hand Off"),
			"BB2016 catch modifiers should include Hand Off, got: " + names(mods));
	}

	// rust: for_rules_bb2020_includes_deflected_pass
	@Test
	public void forRulesBb2020IncludesDeflectedPass() {
		init(GameFixture.createGameState(3, RulesCollection.Rules.BB2020));
		Set<CatchModifier> mods = factory.findModifiers(context(CatchScatterThrowInMode.DEFLECTED));
		assertTrue(names(mods).contains("Deflected Pass"),
			"BB2020 catch modifiers should include Deflected Pass, got: " + names(mods));
	}
}
