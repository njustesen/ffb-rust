package com.fumbbl.ffb.factory;

import com.fumbbl.ffb.FieldCoordinate;
import com.fumbbl.ffb.PlayerAction;
import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.model.Game;
import com.fumbbl.ffb.model.Player;
import com.fumbbl.ffb.model.RosterPlayer;
import com.fumbbl.ffb.modifiers.JumpContext;
import com.fumbbl.ffb.modifiers.JumpModifier;
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
 * Mirror of ffb-rust crates/ffb-mechanics/src/modifiers/jump_modifier_factory.rs tests.
 * Uses the edition factory subclasses directly (mixed for BB2020/BB2025, bb2016 for BB2016),
 * matching the Scanner selection Java performs at runtime.
 */
public class JumpModifierFactoryTest {

	private GameState gameState;
	private Game game;
	private JumpModifierFactory factory;

	@BeforeEach
	void setUp() {
		initMixed(GameFixture.createGameState(3));
	}

	private void initMixed(GameState gs) {
		init(gs, new com.fumbbl.ffb.factory.mixed.JumpModifierFactory());
	}

	private void initBb2016(GameState gs) {
		init(gs, new com.fumbbl.ffb.factory.bb2016.JumpModifierFactory());
	}

	private void init(GameState gs, JumpModifierFactory f) {
		gameState = gs;
		game = gameState.getGame();
		factory = f;
		factory.initialize(game);
		GameFixture.placePlayer(gameState, "home1", 5, 5);
		GameFixture.setActingPlayer(gameState, "home1", PlayerAction.MOVE);
	}

	private Player<?> jumper() {
		return game.getPlayerById("home1");
	}

	private JumpContext context() {
		return new JumpContext(game, jumper(), new FieldCoordinate(5, 5), new FieldCoordinate(6, 5));
	}

	private void addSkill(String playerId, String skillName) {
		((RosterPlayer) game.getPlayerById(playerId)).addSkill(GameFixture.skill(game, skillName));
	}

	private Set<String> names(Set<JumpModifier> modifiers) {
		return modifiers.stream().map(JumpModifier::getName).collect(Collectors.toSet());
	}

	private String forNameOf(JumpModifierFactory f, String name) {
		if (f instanceof com.fumbbl.ffb.factory.mixed.JumpModifierFactory) {
			JumpModifier m = ((com.fumbbl.ffb.factory.mixed.JumpModifierFactory) f).forName(name);
			return m == null ? null : m.getName();
		}
		JumpModifier m = ((com.fumbbl.ffb.factory.bb2016.JumpModifierFactory) f).forName(name);
		return m == null ? null : m.getName();
	}

	// rust: find_registered_modifiers_bb2016_has_only_very_long_legs
	@Test
	public void findRegisteredModifiersBb2016HasOnlyVeryLongLegs() {
		initBb2016(GameFixture.createGameState(3, RulesCollection.Rules.BB2016));
		Set<JumpModifier> mods = game.getModifierAggregator().getJumpModifiers();
		assertEquals(1, mods.size());
		assertEquals("Very Long Legs", mods.iterator().next().getName());
	}

	// rust: find_registered_modifiers_bb2025_has_all_three
	@Test
	public void findRegisteredModifiersBb2025HasAllThree() {
		Set<JumpModifier> mods = game.getModifierAggregator().getJumpModifiers();
		Set<String> names = names(mods);
		assertTrue(names.contains("Very Long Legs"));
		assertTrue(names.contains("Leap"));
		assertTrue(names.contains("Diving Tackle"));
		assertEquals(3, mods.size());
	}

	// rust: for_rules_bb2025_has_tacklezone_modifiers
	@Test
	public void forRulesBb2025HasTacklezoneModifiers() {
		assertNotNull(forNameOf(factory, "1 Tacklezone"),
			"mixed jump collection should carry tacklezone modifiers");
	}

	// rust: for_rules_bb2016_has_no_modifiers
	@Test
	public void forRulesBb2016HasNoModifiers() {
		initBb2016(GameFixture.createGameState(3, RulesCollection.Rules.BB2016));
		assertNull(forNameOf(factory, "1 Tacklezone"),
			"bb2016 jump collection should be empty");
	}

	// rust: minimum_roll_base_is_agility
	@Test
	public void minimumRollBaseIsAgility() {
		assertEquals(3, new com.fumbbl.ffb.mechanics.bb2025.AgilityMechanic()
			.minimumRollJump(jumper(), Collections.emptySet()));
	}

	// rust: minimum_roll_never_below_two
	@Test
	public void minimumRollNeverBelowTwo() {
		((RosterPlayer) jumper()).setAgility(1);
		assertEquals(2, new com.fumbbl.ffb.mechanics.bb2025.AgilityMechanic()
			.minimumRollJump(jumper(), Collections.emptySet()));
	}

	// rust: minimum_roll_adds_modifier_total
	@Test
	public void minimumRollAddsModifierTotal() {
		JumpModifier m = new JumpModifier("1 Tacklezone", 1, ModifierType.TACKLEZONE);
		assertEquals(4, new com.fumbbl.ffb.mechanics.bb2025.AgilityMechanic()
			.minimumRollJump(jumper(), Collections.singleton(m)));
	}

	// rust: find_skill_modifiers_very_long_legs_bb2025_regular
	@Test
	public void findSkillModifiersVeryLongLegsBb2025Regular() {
		addSkill("home1", "Very Long Legs");
		Set<JumpModifier> mods = factory.findModifiers(context());
		assertTrue(mods.stream().anyMatch(m -> "Very Long Legs".equals(m.getName())
				&& m.getType() == ModifierType.REGULAR),
			"BB2025 Very Long Legs should be a REGULAR modifier, got: " + names(mods));
	}

	// rust: find_skill_modifiers_very_long_legs_bb2020_depends_on_sum_applies_when_accumulated_gt_1
	@Test
	public void findSkillModifiersVeryLongLegsBb2020DependsOnSumAppliesWhenAccumulatedGt1() {
		initMixed(GameFixture.createGameState(3, RulesCollection.Rules.BB2020));
		addSkill("home1", "Very Long Legs");

		// No accumulated modifier -> not applied.
		assertTrue(factory.findModifiers(context()).isEmpty());

		// Accumulated > 1 -> applied.
		JumpContext context = context();
		context.setAccumulatedModifiers(2);
		Set<JumpModifier> mods = factory.findModifiers(context);
		assertTrue(names(mods).contains("Very Long Legs"));
	}

	// rust: find_skill_modifiers_leap_bb2025_applies_when_accumulated_gt_1
	@Test
	public void findSkillModifiersLeapBb2025AppliesWhenAccumulatedGt1() {
		addSkill("home1", "Leap");

		// accumulated > 1 -> applies.
		JumpContext context = context();
		context.setAccumulatedModifiers(2);
		assertTrue(names(factory.findModifiers(context)).contains("Leap"));

		// accumulated == 0, count == 0 -> does not apply.
		assertFalse(names(factory.findModifiers(context())).contains("Leap"));
	}

	// rust: find_skill_modifiers_leap_bb2016_no_modifier
	@Test
	public void findSkillModifiersLeapBb2016NoModifier() {
		initBb2016(GameFixture.createGameState(3, RulesCollection.Rules.BB2016));
		addSkill("home1", "Leap");
		JumpContext context = context();
		context.setAccumulatedModifiers(5);
		assertTrue(factory.findModifiers(context).isEmpty(),
			"BB2016 Leap registers no JumpModifier");
	}
}
