package com.fumbbl.ffb.client.state.logic.mixed;

import com.fumbbl.ffb.ClientStateId;
import com.fumbbl.ffb.client.FantasyFootballClient;
import com.fumbbl.ffb.client.state.logic.ClientAction;
import com.fumbbl.ffb.client.state.logic.interaction.ActionContext;
import com.fumbbl.ffb.model.ActingPlayer;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.Answers;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;
import org.mockito.junit.jupiter.MockitoSettings;
import org.mockito.quality.Strictness;

import java.util.Set;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.Mockito.when;

// Mirrors ffb-rust ffb-client crates/ffb-client/src/client/state/logic/mixed/then_i_started_blastin_logic_module.rs
// (Rust: mod tests), one test fn per Rust #[test] that is faithfully reproducible.
//
// SKIPPED (with reasons):
// - find_adjacent_coordinates_excludes_start_by_default / _includes_start_when_requested:
//   these exercise a Rust-only free function reimplementing FieldModel.findAdjacentCoordinates
//   (a gap-fill documented in the Rust source); the real Java method lives on FieldModel, which
//   has no simple/side-effect-free constructor to instantiate here.
// - is_valid_target_false_without_source_coordinate / is_valid_target_true_within_distance_and_standing:
//   Java's isValidTarget(Player, Game) is a `private` method, not accessible from a test in the
//   same package.
// - player_interaction_ignores_without_game: falls through to isValidTarget(player, game)
//   (called internally, so private-ness isn't a blocker), but that method dereferences
//   game.getFieldModel().getPlayerCoordinate(player) as a non-null target coordinate for
//   distanceInSteps(...) -- with a deep-stub Game this is null, causing an NPE rather than the
//   graceful "ignore" Rust produces from client.game() being None.
// - set_up_and_teardown_manage_move_squares: setUp() computes FieldModel.findAdjacentCoordinates
//   over a real player/field graph -- live game state, out of scope.
// - perform_available_action_no_op_without_game: Java's END_MOVE case (unlike Rust's "without
//   game" short-circuit) will actually invoke communication.sendEndTurn(...) under deep-stub
//   defaults (isEndPlayerActionAvailable() defaults to true), which is not an equivalent no-op
//   scenario to what Rust exercises.
// - defender_none_without_defender_id: exercises the real Java Game.getDefender(), but Game's
//   only constructor requires an IFactorySource/FactoryManager graph, out of scope to construct
//   here (unlike Rust's plain Game::new(team, team, rules)).
@ExtendWith(MockitoExtension.class)
@MockitoSettings(strictness = Strictness.LENIENT)
class ThenIStartedBlastinLogicModuleTest {

	@Mock(answer = Answers.RETURNS_DEEP_STUBS)
	FantasyFootballClient client;

	@Test
	void getIdIsThenIStartedBlastin() {
		ThenIStartedBlastinLogicModule module = new ThenIStartedBlastinLogicModule(client);
		assertEquals(ClientStateId.THEN_I_STARTED_BLASTIN, module.getId());
	}

	@Test
	void availableActionsIsEndMoveOnly() {
		ThenIStartedBlastinLogicModule module = new ThenIStartedBlastinLogicModule(client);
		Set<ClientAction> actions = module.availableActions();
		assertEquals(1, actions.size());
		assertTrue(actions.contains(ClientAction.END_MOVE));
	}

	@Test
	void actionContextAddsEndMoveWhenNotActed() {
		ThenIStartedBlastinLogicModule module = new ThenIStartedBlastinLogicModule(client);
		ActingPlayer actingPlayer = new ActingPlayer(null);
		when(client.getGame().getActingPlayer()).thenReturn(actingPlayer);

		ActionContext ctx = module.actionContext(actingPlayer);

		assertTrue(ctx.getActions().contains(ClientAction.END_MOVE));
	}

	@Test
	void actionContextEmptyWhenAlreadyActed() {
		ThenIStartedBlastinLogicModule module = new ThenIStartedBlastinLogicModule(client);
		ActingPlayer actingPlayer = new ActingPlayer(null);
		actingPlayer.setHasMoved(true);
		when(client.getGame().getActingPlayer()).thenReturn(actingPlayer);

		ActionContext ctx = module.actionContext(actingPlayer);

		assertTrue(ctx.getActions().isEmpty());
	}
}
