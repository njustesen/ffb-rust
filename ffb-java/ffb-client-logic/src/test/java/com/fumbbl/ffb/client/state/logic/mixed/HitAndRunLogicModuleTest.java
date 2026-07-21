package com.fumbbl.ffb.client.state.logic.mixed;

import com.fumbbl.ffb.ClientStateId;
import com.fumbbl.ffb.client.state.logic.ClientAction;
import org.junit.jupiter.api.Test;

import java.util.Set;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;

// Mirrors ffb-rust ffb-client crates/ffb-client/src/client/state/logic/mixed/hit_and_run_logic_module.rs
// (Rust: mod tests), one test fn per Rust #[test] that is faithfully reproducible.
//
// SKIPPED (with reasons):
// - action_context_empty_without_skill / action_context_adds_hit_and_run_with_skill:
//   Java's actionContext(ActingPlayer) calls UtilCards.hasUnusedSkillWithProperty(actingPlayer,
//   ...), whose body dereferences actingPlayer.getPlayer(), which itself calls
//   getGame().getPlayerById(getPlayerId()) -- a real ActingPlayer requires a populated Game
//   with a matching Player to avoid an NPE here, out of scope per the "real ActingPlayer/Game
//   object graph" skip rule.
// - field_interaction_ignores_without_move_square / field_peek_invalid_without_move_square:
//   both drive the private isValidField(FieldCoordinate), which dereferences
//   client.getGame().getFieldModel().getMoveSquare(coordinate) -- out of scope.
// - player_peek_reset_for_acting_player / player_interaction_ignores_without_game:
//   both dereference client.getGame().getActingPlayer() unconditionally; the "acting player"
//   branch additionally needs a populated Game -- out of scope.
// - perform_available_action_sends_end_turn / perform_available_action_no_op_without_game:
//   dereferences client.getGame().getTurnMode() unconditionally when the action matches --
//   out of scope for a plain no-op test since it exercises no real branch of interest here
//   without a Game.
class HitAndRunLogicModuleTest {

	// HitAndRunLogicModule extends LogicModule directly with a trivial `super(client)`
	// constructor (no LOGIC_PLUGIN factory resolution), so a null client is safe here.
	private final HitAndRunLogicModule module = new HitAndRunLogicModule(null);

	@Test
	void getIdReturnsHitAndRun() {
		assertEquals(ClientStateId.HIT_AND_RUN, module.getId());
	}

	@Test
	void availableActionsIsHitAndRunOnly() {
		Set<ClientAction> actions = module.availableActions();
		assertEquals(1, actions.size());
		assertTrue(actions.contains(ClientAction.HIT_AND_RUN));
	}
}
