package com.fumbbl.ffb.client.state.logic.bb2020;

import com.fumbbl.ffb.ClientStateId;
import com.fumbbl.ffb.client.FantasyFootballClient;
import com.fumbbl.ffb.client.state.logic.ClientAction;
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
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.when;

// Mirrors ffb-rust ffb-client crates/ffb-client/src/client/state/logic/bb2020/throw_keg_logic_module.rs
// (Rust: mod tests), one test fn per Rust #[test] that is faithfully reproducible.
//
// SKIPPED (with reasons):
// - is_valid_target_false_without_positions / is_valid_target_true_for_close_standing_opponent:
//   Java's isValidTarget(Player, Game) is `private`, not accessible from this test class
//   (Rust's equivalent is a free fn callable only because the test module lives inside the
//   same source file); it would also require a live populated Game/Team/Player graph for the
//   "true" case.
// - set_up_adds_move_squares_around_acting_player: setUp() calls
//   FieldModel.findAdjacentCoordinates(...), real adjacency logic over a live field model --
//   out of scope.
// - player_interaction_ignores_without_game: client.getGame() is called unconditionally and
//   NPEs on a null deep-stub return -- cannot express "no game" in Java.
//
// ADAPTED: teardown_clears_move_squares is ported via Mockito verification (that
// fieldModel.clearMoveSquares() was invoked) rather than inspecting FieldModel's internal
// move-square list, since the field model here is a deep-stub mock, not a real populated one.
@ExtendWith(MockitoExtension.class)
@MockitoSettings(strictness = Strictness.LENIENT)
class ThrowKegLogicModuleTest {

	@Mock(answer = Answers.RETURNS_DEEP_STUBS)
	FantasyFootballClient client;

	@Test
	void getIdReturnsThrowKeg() {
		ThrowKegLogicModule module = new ThrowKegLogicModule(client);
		assertEquals(ClientStateId.THROW_KEG, module.getId());
	}

	@Test
	void availableActionsMatchesJava() {
		ThrowKegLogicModule module = new ThrowKegLogicModule(client);
		Set<ClientAction> actions = module.availableActions();
		assertTrue(actions.contains(ClientAction.END_MOVE));
		assertEquals(9, actions.size());
	}

	@Test
	void isEndPlayerActionAvailableTrueWithoutActing() {
		when(client.getGame().getActingPlayer().hasActed()).thenReturn(false);
		ThrowKegLogicModule module = new ThrowKegLogicModule(client);

		assertTrue(module.isEndPlayerActionAvailable());
	}

	@Test
	void teardownClearsMoveSquares() {
		ThrowKegLogicModule module = new ThrowKegLogicModule(client);

		module.teardown();

		verify(client.getGame().getFieldModel()).clearMoveSquares();
	}
}
