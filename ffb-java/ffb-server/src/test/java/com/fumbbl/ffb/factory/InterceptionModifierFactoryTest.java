package com.fumbbl.ffb.factory;

import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.mechanics.PassResult;
import com.fumbbl.ffb.model.Game;
import com.fumbbl.ffb.model.Player;
import com.fumbbl.ffb.model.RosterPlayer;
import com.fumbbl.ffb.modifiers.InterceptionContext;
import com.fumbbl.ffb.modifiers.InterceptionModifier;
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
 * Mirror of ffb-rust crates/ffb-mechanics/src/modifiers/interception_modifier_factory.rs tests.
 * The factory is initialized against a GameFixture game (players start in the reserve box,
 * so only explicitly placed players project tackle zones).
 */
public class InterceptionModifierFactoryTest {

	private GameState gameState;
	private Game game;
	private InterceptionModifierFactory factory;

	@BeforeEach
	void setUp() {
		init(GameFixture.createGameState(3, RulesCollection.Rules.BB2016));
	}

	private void init(GameState gs) {
		gameState = gs;
		game = gameState.getGame();
		factory = new InterceptionModifierFactory();
		factory.initialize(game);
		GameFixture.placePlayer(gameState, "home1", 5, 5);
		// The bb2025 "Thrower has Stunty" modifier dereferences game.getThrower()
		// unconditionally — a thrower must always be set during an interception.
		game.setThrowerId("away2");
	}

	private Player<?> interceptor() {
		return game.getPlayerById("home1");
	}

	private InterceptionContext context(PassResult passResult) {
		return new InterceptionContext(game, interceptor(), passResult, false);
	}

	private void addSkill(String playerId, String skillName) {
		((RosterPlayer) game.getPlayerById(playerId)).addSkill(GameFixture.skill(game, skillName));
	}

	private Set<String> names(Set<InterceptionModifier> modifiers) {
		return modifiers.stream().map(InterceptionModifier::getName).collect(Collectors.toSet());
	}

	// rust: for_rules_bb2016_returns_factory
	@Test
	public void forRulesBb2016ReturnsFactory() {
		assertNotNull(factory.forName("1 Tacklezone"),
			"BB2016 interception collection should be loaded");
	}

	// rust: for_rules_bb2025_returns_factory
	@Test
	public void forRulesBb2025ReturnsFactory() {
		init(GameFixture.createGameState(3));
		assertNotNull(factory.forName("1 Tacklezone"),
			"BB2025 interception collection should be loaded");
	}

	// rust: no_modifiers_for_isolated_interceptor
	@Test
	public void noModifiersForIsolatedInterceptor() {
		Set<InterceptionModifier> mods = factory.findModifiers(context(PassResult.ACCURATE));
		assertTrue(mods.isEmpty(), "isolated interceptor should have no modifiers, got: " + names(mods));
	}

	// rust: tacklezone_modifier_applied_when_marked
	@Test
	public void tacklezoneModifierAppliedWhenMarked() {
		GameFixture.placePlayer(gameState, "away1", 6, 5);
		Set<InterceptionModifier> mods = factory.findModifiers(context(PassResult.ACCURATE));
		assertTrue(mods.stream().anyMatch(m -> m.getType() == ModifierType.TACKLEZONE),
			"expected TACKLEZONE modifier when adjacent opponent has TZ");
	}

	// rust: ignore_tacklezones_when_catching_skips_tz_modifiers
	@Test
	public void ignoreTacklezonesWhenCatchingSkipsTzModifiers() {
		addSkill("home1", "Nerves of Steel");
		GameFixture.placePlayer(gameState, "away1", 6, 5);
		Set<InterceptionModifier> mods = factory.findModifiers(context(PassResult.ACCURATE));
		assertFalse(mods.stream().anyMatch(m -> m.getType() == ModifierType.TACKLEZONE),
			"TZ modifier should be skipped when player ignores TZs when catching");
	}

	// rust: disturbing_presence_modifier_selected_by_count
	@Test
	public void disturbingPresenceModifierSelectedByCount() {
		addSkill("away1", "Disturbing Presence");
		GameFixture.placePlayer(gameState, "away1", 6, 5);
		Set<InterceptionModifier> mods = factory.findModifiers(context(PassResult.ACCURATE));
		List<InterceptionModifier> dpMods = mods.stream()
			.filter(m -> m.getType() == ModifierType.DISTURBING_PRESENCE).collect(Collectors.toList());
		assertEquals(1, dpMods.size(), "exactly one DP modifier should be present");
		assertEquals(1, dpMods.get(0).getModifier());
	}

	// rust: minimum_roll_bb2016_agility_3_no_modifiers
	@Test
	public void minimumRollBb2016Agility3NoModifiers() {
		assertEquals(6, new com.fumbbl.ffb.mechanics.bb2016.AgilityMechanic()
			.minimumRollInterception(interceptor(), Collections.emptySet()));
	}

	// rust: minimum_roll_bb2016_agility_6_no_modifiers
	@Test
	public void minimumRollBb2016Agility6NoModifiers() {
		((RosterPlayer) interceptor()).setAgility(6);
		assertEquals(3, new com.fumbbl.ffb.mechanics.bb2016.AgilityMechanic()
			.minimumRollInterception(interceptor(), Collections.emptySet()));
	}

	// rust: minimum_roll_bb2016_adds_modifier
	@Test
	public void minimumRollBb2016AddsModifier() {
		InterceptionModifier m = new InterceptionModifier("1 Tacklezone", 1, ModifierType.TACKLEZONE);
		assertEquals(7, new com.fumbbl.ffb.mechanics.bb2016.AgilityMechanic()
			.minimumRollInterception(interceptor(), Collections.singleton(m)));
	}

	// rust: minimum_roll_bb2016_never_below_two
	@Test
	public void minimumRollBb2016NeverBelowTwo() {
		((RosterPlayer) interceptor()).setAgility(6);
		InterceptionModifier m = new InterceptionModifier("bonus", -5, ModifierType.REGULAR);
		assertEquals(2, new com.fumbbl.ffb.mechanics.bb2016.AgilityMechanic()
			.minimumRollInterception(interceptor(), Collections.singleton(m)));
	}

	// rust: find_skill_modifiers_extra_arms_applies
	@Test
	public void findSkillModifiersExtraArmsApplies() {
		init(GameFixture.createGameState(3));
		addSkill("home1", "Extra Arms");
		Set<InterceptionModifier> mods = factory.findModifiers(context(PassResult.ACCURATE));
		InterceptionModifier extraArms = mods.stream().filter(m -> "Extra Arms".equals(m.getName()))
			.findFirst().orElse(null);
		assertNotNull(extraArms, "Extra Arms should apply");
		assertEquals(-1, extraArms.getModifier());
	}

	// rust: find_skill_modifiers_very_long_legs_bb2016_is_minus_one
	@Test
	public void findSkillModifiersVeryLongLegsBb2016IsMinusOne() {
		addSkill("home1", "Very Long Legs");
		Set<InterceptionModifier> mods = factory.findModifiers(context(PassResult.ACCURATE));
		InterceptionModifier vll = mods.stream().filter(m -> "Very Long Legs".equals(m.getName()))
			.findFirst().orElse(null);
		assertNotNull(vll);
		assertEquals(-1, vll.getModifier());
	}

	// rust: find_skill_modifiers_very_long_legs_bb2025_is_minus_two
	@Test
	public void findSkillModifiersVeryLongLegsBb2025IsMinusTwo() {
		init(GameFixture.createGameState(3));
		addSkill("home1", "Very Long Legs");
		Set<InterceptionModifier> mods = factory.findModifiers(context(PassResult.ACCURATE));
		InterceptionModifier vll = mods.stream().filter(m -> "Very Long Legs".equals(m.getName()))
			.findFirst().orElse(null);
		assertNotNull(vll);
		assertEquals(-2, vll.getModifier());
	}

	// rust: find_skill_modifiers_nerves_of_steel_marker
	@Test
	public void findSkillModifiersNervesOfSteelMarker() {
		init(GameFixture.createGameState(3));
		addSkill("home1", "Nerves of Steel");
		Set<InterceptionModifier> mods = factory.findModifiers(context(PassResult.ACCURATE));
		InterceptionModifier nos = mods.stream().filter(m -> "Nerves of Steel".equals(m.getName()))
			.findFirst().orElse(null);
		assertNotNull(nos, "Nerves of Steel marker should be present");
		assertEquals(0, nos.getModifier());
		assertEquals("0 tackle zones due to Nerves of Steel", nos.getReportString());
		assertTrue(nos.isModifierIncluded());
	}
}
