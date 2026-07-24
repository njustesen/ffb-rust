package com.fumbbl.ffb.client.state;

import com.fumbbl.ffb.ClientStateId;
import com.fumbbl.ffb.FieldCoordinate;
import com.fumbbl.ffb.client.FantasyFootballClient;
import com.fumbbl.ffb.client.state.logic.LogicModule;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;
import org.mockito.junit.jupiter.MockitoSettings;
import org.mockito.quality.Strictness;

import static org.junit.jupiter.api.Assertions.assertDoesNotThrow;
import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertNull;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.when;

/**
 * Port of the {@code #[cfg(test)]} module in
 * {@code ffb-rust/crates/ffb-client/src/client/state/client_state.rs}, against the real abstract
 * {@link ClientState} base. Exercised through a minimal test-local concrete subclass (the real
 * concrete states live in the AWT ffb-client module); the logic module is a Mockito mock so the
 * enter/leave/end-turn delegation and getId delegation are verifiable.
 *
 * <p>Rust tests pruned rather than ported (kept the suites 1:1):
 * {@code icon_progress_lifecycle_tracks_showing_flag} (DialogProgressBar is Swing UI, not
 * headless); {@code get_logic_module_mut_allows_mutation} (Rust interior-mutability plumbing); and
 * {@code team_and_player_imports_are_exercised_by_available_actions} (a Rust import/compile sanity
 * check, not a behavioral assertion).
 */
@ExtendWith(MockitoExtension.class)
@MockitoSettings(strictness = Strictness.LENIENT)
class ClientStateTest {

	@Mock
	FantasyFootballClient client;

	@Mock
	LogicModule logicModule;

	private StubState state;

	/** Minimal concrete ClientState: UI drawSelectSquare is a no-op; exposes the protected coordinate. */
	private static final class StubState extends ClientState<LogicModule, FantasyFootballClient> {
		StubState(FantasyFootballClient client, LogicModule logicModule) {
			super(client, logicModule);
		}

		@Override
		protected void drawSelectSquare() {
		}

		FieldCoordinate exposedCoordinate() {
			return fSelectSquareCoordinate;
		}
	}

	@BeforeEach
	void setUp() {
		state = new StubState(client, logicModule);
	}

	// rust: enter_state_calls_logic_module_set_up_then_own_set_up
	@Test
	void enterStateCallsLogicModuleSetUp() {
		state.enterState();
		verify(logicModule).setUp();
	}

	// rust: leave_state_calls_own_tear_down_then_logic_module_teardown
	@Test
	void leaveStateCallsLogicModuleTeardown() {
		state.leaveState();
		verify(logicModule).teardown();
	}

	// rust: end_turn_calls_logic_module_end_turn_then_post_end_turn
	@Test
	void endTurnCallsLogicModuleEndTurn() {
		state.endTurn();
		verify(logicModule).endTurn();
	}

	// rust: get_id_delegates_to_logic_module
	@Test
	void getIdDelegatesToLogicModule() {
		when(logicModule.getId()).thenReturn(ClientStateId.SELECT_PLAYER);
		assertEquals(ClientStateId.SELECT_PLAYER, state.getId());
	}

	// rust: hide_select_square_clears_coordinate
	@Test
	void hideSelectSquareClearsCoordinate() {
		state.showSelectSquare(new FieldCoordinate(1, 1));
		assertEquals(new FieldCoordinate(1, 1), state.exposedCoordinate());
		state.hideSelectSquare();
		assertNull(state.exposedCoordinate());
	}

	// rust: show_select_square_with_none_does_not_set_coordinate
	@Test
	void showSelectSquareWithNullDoesNotSetCoordinate() {
		state.showSelectSquare(null);
		assertNull(state.exposedCoordinate());
	}

	// rust: show_select_square_with_some_sets_coordinate
	@Test
	void showSelectSquareWithSomeSetsCoordinate() {
		state.showSelectSquare(new FieldCoordinate(3, 4));
		assertEquals(new FieldCoordinate(3, 4), state.exposedCoordinate());
	}

	// rust: action_key_pressed_default_is_false
	@Test
	void actionKeyPressedDefaultIsFalse() {
		assertFalse(state.actionKeyPressed(null));
	}

	// rust: drag_drop_predicates_default_to_true
	@Test
	void dragDropPredicatesDefaultToTrue() {
		FieldCoordinate coord = new FieldCoordinate(0, 0);
		assertTrue(state.isInitDragAllowed(coord));
		assertTrue(state.isDragAllowed(coord));
		assertTrue(state.isDropAllowed(coord));
	}

	// rust: reinitialize_local_state_is_a_no_op_without_dialog
	@Test
	void reinitializeLocalStateIsNoOpWithoutDialog() {
		assertDoesNotThrow(() -> state.reinitializeLocalState());
	}
}
