package com.fumbbl.ffb.factory;

import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.model.Game;
import com.fumbbl.ffb.model.Player;
import com.fumbbl.ffb.model.RosterPlayer;
import com.fumbbl.ffb.modifiers.ModifierType;
import com.fumbbl.ffb.modifiers.RightStuffModifier;
import com.fumbbl.ffb.server.GameState;
import com.fumbbl.ffb.server.fixture.GameFixture;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import java.util.Collections;
import java.util.Set;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNotNull;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirror of ffb-rust crates/ffb-mechanics/src/modifiers/right_stuff_modifier_factory.rs tests.
 */
public class RightStuffModifierFactoryTest {

	private GameState gameState;
	private Game game;
	private RightStuffModifierFactory factory;

	@BeforeEach
	void setUp() {
		init(GameFixture.createGameState(3));
	}

	private void init(GameState gs) {
		gameState = gs;
		game = gameState.getGame();
		factory = new RightStuffModifierFactory();
		factory.initialize(game);
	}

	private Player<?> player() {
		return game.getPlayerById("home1");
	}

	// rust: find_registered_modifiers_bb2016_has_swoop
	@Test
	public void findRegisteredModifiersBb2016HasSwoop() {
		init(GameFixture.createGameState(3, RulesCollection.Rules.BB2016));
		Set<RightStuffModifier> mods = game.getModifierAggregator().getRightStuffModifiers();
		assertEquals(1, mods.size());
		assertEquals("Swoop", mods.iterator().next().getName());
	}

	// rust: find_registered_modifiers_bb2025_is_empty
	@Test
	public void findRegisteredModifiersBb2025IsEmpty() {
		assertTrue(game.getModifierAggregator().getRightStuffModifiers().isEmpty());
	}

	// rust: minimum_roll_base_is_agility
	@Test
	public void minimumRollBaseIsAgility() {
		assertEquals(3, new com.fumbbl.ffb.mechanics.bb2025.AgilityMechanic()
			.minimumRollRightStuff(player(), Collections.emptySet()));
	}

	// rust: minimum_roll_never_below_two
	@Test
	public void minimumRollNeverBelowTwo() {
		((RosterPlayer) player()).setAgility(1);
		assertEquals(2, new com.fumbbl.ffb.mechanics.bb2025.AgilityMechanic()
			.minimumRollRightStuff(player(), Collections.emptySet()));
	}

	// rust: minimum_roll_adds_modifier_total
	@Test
	public void minimumRollAddsModifierTotal() {
		RightStuffModifier m = new RightStuffModifier("Subpar Throw", 1, ModifierType.REGULAR);
		assertEquals(4, new com.fumbbl.ffb.mechanics.bb2025.AgilityMechanic()
			.minimumRollRightStuff(player(), Collections.singleton(m)));
	}

	// rust: for_rules_bb2025_has_pass_result_modifiers
	@Test
	public void forRulesBb2025HasPassResultModifiers() {
		assertNotNull(factory.forName("Subpar Throw"));
		assertNotNull(factory.forName("Fumbled Throw"));
	}

	// rust: for_rules_bb2016_has_kick_range_modifiers
	@Test
	public void forRulesBb2016HasKickRangeModifiers() {
		init(GameFixture.createGameState(3, RulesCollection.Rules.BB2016));
		assertNotNull(factory.forName("Medium Kick"));
		assertNotNull(factory.forName("Long Kick"));
	}
}
