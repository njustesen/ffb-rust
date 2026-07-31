package com.fumbbl.ffb.factory;

import com.fumbbl.ffb.FieldCoordinate;
import com.fumbbl.ffb.PlayerAction;
import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.model.Game;
import com.fumbbl.ffb.model.RosterPlayer;
import com.fumbbl.ffb.modifiers.DodgeContext;
import com.fumbbl.ffb.modifiers.DodgeModifier;
import com.fumbbl.ffb.modifiers.ModifierType;
import com.fumbbl.ffb.server.GameState;
import com.fumbbl.ffb.server.fixture.GameFixture;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import java.util.Collections;
import java.util.Set;
import java.util.stream.Collectors;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertNotNull;
import static org.junit.jupiter.api.Assertions.assertNull;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirror of ffb-rust crates/ffb-mechanics/src/modifiers/dodge_modifier_factory.rs tests.
 * The factory is initialized against a GameFixture game (players start in the reserve box,
 * so only explicitly placed players project tackle zones).
 */
public class DodgeModifierFactoryTest {

	private GameState gameState;
	private Game game;
	private DodgeModifierFactory factory;

	@BeforeEach
	void setUp() {
		gameState = GameFixture.createGameState(3);
		init(gameState);
	}

	private void init(GameState gs) {
		gameState = gs;
		game = gameState.getGame();
		factory = new DodgeModifierFactory();
		factory.initialize(game);
		GameFixture.placePlayer(gameState, "home1", 5, 5);
		GameFixture.setActingPlayer(gameState, "home1", PlayerAction.MOVE);
	}

	private DodgeContext context(int targetX, int targetY) {
		return new DodgeContext(game, game.getActingPlayer(), new FieldCoordinate(5, 5),
			new FieldCoordinate(targetX, targetY));
	}

	private DodgeContext contextWithBreakTackle(int targetX, int targetY) {
		return new DodgeContext(game, game.getActingPlayer(), new FieldCoordinate(5, 5),
			new FieldCoordinate(targetX, targetY), true);
	}

	private void addSkill(String playerId, String skillName) {
		((RosterPlayer) game.getPlayerById(playerId)).addSkill(GameFixture.skill(game, skillName));
	}

	private Set<String> names(Set<DodgeModifier> modifiers) {
		return modifiers.stream().map(DodgeModifier::getName).collect(Collectors.toSet());
	}

	// rust: for_name_returns_tackle_zone_modifier
	@Test
	public void forNameReturnsTackleZoneModifier() {
		assertNotNull(factory.forName("1 Tacklezone"));
		assertNotNull(factory.forName("8 Tacklezones"));
		assertNull(factory.forName("NonExistent"));
	}

	// rust: no_opponents_no_modifiers
	@Test
	public void noOpponentsNoModifiers() {
		Set<DodgeModifier> mods = factory.findModifiers(context(6, 5));
		assertTrue(mods.isEmpty(), "No opponents should yield no dodge modifiers");
	}

	// rust: find_applicable_includes_prehensile_tail_modifier_for_marked_opponent
	@Test
	public void findModifiersIncludesPrehensileTailModifierForMarkedOpponent() {
		addSkill("away1", "Prehensile Tail");
		GameFixture.placePlayer(gameState, "away1", 6, 5);
		Set<DodgeModifier> mods = factory.findModifiers(context(7, 5));
		assertTrue(
			mods.stream().anyMatch(m -> m.getType() == ModifierType.PREHENSILE_TAIL
				&& "1 Prehensile Tail".equals(m.getName())),
			"expected a '1 Prehensile Tail' modifier, got: " + names(mods));
	}

	// rust: minimum_roll_no_modifiers
	@Test
	public void minimumRollNoModifiers() {
		assertEquals(3, new com.fumbbl.ffb.mechanics.bb2025.AgilityMechanic()
			.minimumRoll(3, Collections.emptySet()));
	}

	// rust: minimum_roll_with_one_tackle_zone
	@Test
	public void minimumRollWithOneTackleZone() {
		DodgeModifier m = new DodgeModifier("1 Tacklezone", 1, ModifierType.TACKLEZONE);
		assertEquals(4, new com.fumbbl.ffb.mechanics.bb2025.AgilityMechanic()
			.minimumRoll(3, Collections.singleton(m)));
	}

	// rust: minimum_roll_never_below_two
	@Test
	public void minimumRollNeverBelowTwo() {
		assertEquals(2, new com.fumbbl.ffb.mechanics.bb2025.AgilityMechanic()
			.minimumRoll(1, Collections.emptySet()));
	}

	// rust: find_skill_modifiers_two_heads_applies_in_bb2025
	@Test
	public void findSkillModifiersTwoHeadsAppliesInBb2025() {
		addSkill("home1", "Two Heads");
		Set<DodgeModifier> mods = factory.findModifiers(context(6, 5));
		DodgeModifier twoHeads = mods.stream().filter(m -> "Two Heads".equals(m.getName()))
			.findFirst().orElse(null);
		assertNotNull(twoHeads, "Two Heads should apply");
		assertEquals(-1, twoHeads.getModifier());
	}

	// rust: find_skill_modifiers_titchy_applies_in_bb2016
	@Test
	public void findSkillModifiersTitchyAppliesInBb2016() {
		init(GameFixture.createGameState(3, RulesCollection.Rules.BB2016));
		addSkill("home1", "Titchy");
		Set<DodgeModifier> mods = factory.findModifiers(context(6, 5));
		assertTrue(names(mods).contains("Titchy"), "Titchy should appear in BB2016");
	}

	// rust: find_skill_modifiers_titchy_applies_in_bb2025
	// (mixed.Titchy is registered for BB2020+BB2025 with the same unconditional -1 modifier;
	// the original Rust translation wrongly gated Titchy on BB2016 — fixed against this test.)
	@Test
	public void findSkillModifiersTitchyAppliesInBb2025() {
		addSkill("home1", "Titchy");
		Set<DodgeModifier> mods = factory.findModifiers(context(6, 5));
		DodgeModifier titchy = mods.stream().filter(m -> "Titchy".equals(m.getName()))
			.findFirst().orElse(null);
		assertNotNull(titchy, "Titchy should appear in BB2025 (mixed.Titchy)");
		assertEquals(-1, titchy.getModifier());
	}

	// rust: find_skill_modifiers_break_tackle_applies_when_use_break_tackle_flag_set
	@Test
	public void findSkillModifiersBreakTackleAppliesWhenUseBreakTackleFlagSet() {
		init(GameFixture.createGameState(3, RulesCollection.Rules.BB2016));
		addSkill("home1", "Break Tackle");
		Set<DodgeModifier> mods = factory.findModifiers(contextWithBreakTackle(6, 5));
		DodgeModifier breakTackle = mods.stream().filter(m -> "Break Tackle".equals(m.getName()))
			.findFirst().orElse(null);
		assertNotNull(breakTackle, "Break Tackle modifier should appear when useBreakTackle=true");
		assertTrue(breakTackle.isUseStrength(), "Break Tackle modifier should have useStrength=true");
	}

	// rust: find_skill_modifiers_break_tackle_bb2020_strength_tiers
	@Test
	public void findSkillModifiersBreakTackleBb2020StrengthTiers() {
		init(GameFixture.createGameState(3, RulesCollection.Rules.BB2020));
		addSkill("home1", "Break Tackle");

		// default fixture strength is 3 -> ST 4- tier (-1)
		Set<DodgeModifier> mods = factory.findModifiers(contextWithBreakTackle(6, 5));
		DodgeModifier lowTier = mods.stream().filter(m -> "Break Tackle ST 4-".equals(m.getName()))
			.findFirst().orElse(null);
		assertNotNull(lowTier, "ST 3 player should get the ST 4- tier (-1)");
		assertEquals(-1, lowTier.getModifier());
		assertFalse(names(mods).contains("Break Tackle ST 5+"));

		((RosterPlayer) game.getPlayerById("home1")).setStrength(5);
		mods = factory.findModifiers(contextWithBreakTackle(6, 5));
		DodgeModifier highTier = mods.stream().filter(m -> "Break Tackle ST 5+".equals(m.getName()))
			.findFirst().orElse(null);
		assertNotNull(highTier, "ST 5 player should get the ST 5+ tier (-2)");
		assertEquals(-2, highTier.getModifier());
		assertFalse(names(mods).contains("Break Tackle ST 4-"));
	}

	// rust: find_skill_modifiers_no_skill_mods_without_skills
	@Test
	public void findSkillModifiersNoSkillModsWithoutSkills() {
		Set<DodgeModifier> mods = factory.findModifiers(context(6, 5));
		assertTrue(mods.isEmpty(), "No skill modifiers without skills");
	}

	// rust: find_registered_modifiers_bb2025_matches_java_aggregator
	@Test
	public void findRegisteredModifiersBb2025MatchesJavaAggregator() {
		Set<DodgeModifier> mods = game.getModifierAggregator().getDodgeModifiers();
		Set<String> names = names(mods);
		assertTrue(names.contains("Two Heads"));
		assertTrue(names.contains("Titchy"));
		assertTrue(names.contains("Diving Tackle"));
		assertTrue(names.contains("Break Tackle ST 5+"));
		assertTrue(names.contains("Break Tackle ST 4"));
		assertTrue(names.contains("Break Tackle ST 3-"));
		assertEquals(6, mods.size());
	}

	// rust: find_registered_modifiers_bb2016_has_all_five
	@Test
	public void findRegisteredModifiersBb2016HasAllFive() {
		init(GameFixture.createGameState(3, RulesCollection.Rules.BB2016));
		Set<DodgeModifier> mods = game.getModifierAggregator().getDodgeModifiers();
		Set<String> names = names(mods);
		assertTrue(names.contains("Two Heads"));
		assertTrue(names.contains("Titchy"));
		assertTrue(names.contains("Stunty"));
		assertTrue(names.contains("Break Tackle"));
		assertTrue(names.contains("Diving Tackle"));
		assertEquals(5, mods.size());
	}
}
