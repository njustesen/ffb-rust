package com.fumbbl.ffb.client.state.logic.mixed;

import com.fumbbl.ffb.ClientStateId;
import com.fumbbl.ffb.client.state.logic.ClientAction;
import com.fumbbl.ffb.client.state.logic.Influences;
import com.fumbbl.ffb.client.state.logic.interaction.ActionContext;
import com.fumbbl.ffb.model.ActingPlayer;
import org.junit.jupiter.api.Test;

import java.util.Set;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

// Mirrors ffb-rust ffb-client crates/ffb-client/src/client/state/logic/mixed/furious_outburst_logic_module.rs
// (Rust: mod tests), one test fn per Rust #[test] that is faithfully reproducible.
//
// SKIPPED (with reasons):
// - field_interaction_ignores_without_move_square / field_peek_invalid_without_move_square:
//   both drive the private isEligible(FieldCoordinate), which dereferences
//   client.getGame().getFieldModel().getMoveSquare(coordinate) -- out of scope per the
//   "live game state to drive fieldInteraction()/fieldPeek()" skip rule.
// - player_interaction_ignores_without_game / player_interaction_selects_action_for_acting_player:
//   Java's playerInteraction(Player<?>) unconditionally dereferences
//   client.getGame().getActingPlayer() with no null-game guard; the "selects action" branch
//   additionally requires a live ActingPlayer whose getPlayer() resolves against a populated
//   Game -- out of scope.
// - perform_available_action_is_no_op: Java's performAvailableAction guards on
//   `if (pPlayer != null)` -- there is no equivalent "no game" no-op to assert; not worth a
//   trivial null-player test here since it exercises nothing beyond the switch default.
class FuriousOutburstLogicModuleTest {

	// FuriousOutburstLogicModule's constructor is the trivial `super(pClient)` (no LOGIC_PLUGIN
	// factory resolution), so a null client is safe as long as we never touch client-dependent
	// methods.
	private final FuriousOutburstLogicModule module = new FuriousOutburstLogicModule(null);

	@Test
	void getIdReturnsFuriousOutburst() {
		assertEquals(ClientStateId.FURIOUS_OUTBURST, module.getId());
	}

	@Test
	void availableActionsIsEndMoveOnly() {
		Set<ClientAction> actions = module.availableActions();
		assertEquals(1, actions.size());
		assertTrue(actions.contains(ClientAction.END_MOVE));
	}

	// Rust: action_context_adds_influence_when_acted
	@Test
	void actionContextAddsInfluenceWhenActed() {
		ActingPlayer actingPlayer = new ActingPlayer(null);
		actingPlayer.setForgone(true); // simplest field that makes hasActed() true without a Game

		ActionContext ctx = module.actionContext(actingPlayer);

		assertTrue(ctx.getActions().contains(ClientAction.END_MOVE));
		assertTrue(ctx.getInfluences().contains(Influences.HAS_ACTED));
	}

	// Rust: action_context_no_influence_without_acted
	@Test
	void actionContextNoInfluenceWithoutActed() {
		ActingPlayer actingPlayer = new ActingPlayer(null);

		ActionContext ctx = module.actionContext(actingPlayer);

		assertFalse(ctx.getInfluences().contains(Influences.HAS_ACTED));
	}
}
