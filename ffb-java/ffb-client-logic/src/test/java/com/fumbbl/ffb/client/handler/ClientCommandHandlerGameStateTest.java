package com.fumbbl.ffb.client.handler;

import com.fumbbl.ffb.client.FantasyFootballClient;
import com.fumbbl.ffb.client.dialog.IDialog;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;

import static org.mockito.Mockito.verify;

// Mirrors ffb-rust ffb-client crates/ffb-client/src/client/handler/client_command_handler_game_state.rs
// (Rust: mod tests), one test fn per Rust #[test] that is faithfully reproducible.
//
// SKIPPED (with reasons):
//
// - apply_game_state_keeps_existing_player_markers_on_initial_state: Rust's
//   `apply_game_state` is a thin wrapper that takes `existing_game`/`incoming_game`/
//   `client_mode`/`is_manual_marking` as explicit parameters and delegates entirely to
//   `SubHandlerGameStateMarking::handle_net_command`. The real Java class has no
//   standalone `applyGameState` method -- that sub-handler delegation
//   (`subHandler.handleNetCommand(gameStateCommand)`) is inlined at the top of
//   `handleNetCommand` together with real IconCache HTTP/ImageIO calls, a
//   `ForkJoinPool` of icon-preloading `LoadTask`s, `UtilClientThrowTeamMate` (which
//   dereferences `game.getActingPlayer()`/`game.getDefender()` unconditionally) and a
//   `SwingUtilities.invokeAndWait` UI refresh. Reaching the marker-copy behavior
//   through the real `handleNetCommand` would require executing or unfaithfully
//   mocking that real network/AWT code, which is not a reliable, faithful mockable
//   seam. The underlying marker-copy scenario (mode PLAYER, initial state) is already
//   fully covered 1:1 by `SubHandlerGameStateMarkingTest.testAutomaticPlayerInitialGameState`
//   (already ported), which is the real Java counterpart `SubHandlerGameStateMarking`
//   that Rust's `apply_game_state` wraps.
// - handle_net_command_returns_true_for_matching_command: Rust's `handle_net_command`
//   is a no-op match arm (the sub-handler delegation is only a `// java:` comment, not
//   implemented) that always returns `true` regardless of command type. The real Java
//   `handleNetCommand` is not a thin no-op -- invoking it for real drives the same
//   IconCache/HTTP/ImageIO/ForkJoinPool/Swing code paths described above with no
//   faithful mockable seam, so asserting `true` here would not be testing an equivalent
//   scenario.
// - handle_net_command_is_a_no_op_for_a_mismatched_command_type: Java casts
//   unconditionally; wrong type throws CCE, not a no-op.
@ExtendWith(MockitoExtension.class)
class ClientCommandHandlerGameStateTest {

	@Mock
	private FantasyFootballClient client;

	@Test
	void dialogClosedDoesNotPanic() {
		ClientCommandHandlerGameState handler = new ClientCommandHandlerGameState(client);

		handler.dialogClosed((IDialog) null);

		verify(client).exitClient();
	}
}
