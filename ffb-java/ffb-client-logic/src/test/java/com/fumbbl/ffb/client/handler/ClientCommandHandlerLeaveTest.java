package com.fumbbl.ffb.client.handler;

import com.fumbbl.ffb.ClientMode;
import com.fumbbl.ffb.client.ClientData;
import com.fumbbl.ffb.client.FantasyFootballClient;
import com.fumbbl.ffb.client.StatusReport;
import com.fumbbl.ffb.client.UserInterface;
import com.fumbbl.ffb.client.ui.LogComponent;
import com.fumbbl.ffb.net.NetCommandId;
import com.fumbbl.ffb.net.commands.ServerCommandLeave;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.Answers;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;
import org.mockito.junit.jupiter.MockitoSettings;
import org.mockito.quality.Strictness;

import java.util.Collections;
import java.util.List;

import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.mockito.Mockito.never;
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.verifyNoInteractions;

/**
 * Mirrors Rust {@code client_command_handler_leave.rs} tests. Rust's
 * {@code should_stop_turn_timer} pure helper has no Java equivalent (Java inlines the check);
 * the port asserts the same condition via the {@link ClientData#setTurnTimerStopped} seam.
 */
@ExtendWith(MockitoExtension.class)
@MockitoSettings(strictness = Strictness.LENIENT)
class ClientCommandHandlerLeaveTest {

	@Mock(answer = Answers.RETURNS_DEEP_STUBS)
	private FantasyFootballClient client;

	private ClientCommandHandlerLeave handler;

	@Test
	void getIdReturnsServerLeave() {
		handler = new ClientCommandHandlerLeave(client);
		assertEquals(NetCommandId.SERVER_LEAVE, handler.getId());
	}

	@Test
	void handleNetCommandShortCircuitsWhenQueuing() {
		handler = new ClientCommandHandlerLeave(client);
		ServerCommandLeave cmd = new ServerCommandLeave("Bob", ClientMode.PLAYER, Collections.emptyList());

		assertTrue(handler.handleNetCommand(cmd, ClientCommandHandlerMode.QUEUING));

		verifyNoInteractions(client);
	}

	@Test
	void handleNetCommandStopsTurnTimerForPlayerMode() {
		handler = new ClientCommandHandlerLeave(client);
		ClientData clientData = client.getClientData();
		ServerCommandLeave cmd = new ServerCommandLeave("Bob", ClientMode.PLAYER, Collections.emptyList());

		assertTrue(handler.handleNetCommand(cmd, ClientCommandHandlerMode.PLAYING));

		verify(clientData).setTurnTimerStopped(true);
	}

	@Test
	void handleNetCommandDoesNotStopTurnTimerForSpectatorOrReplayMode() {
		handler = new ClientCommandHandlerLeave(client);
		ClientData clientData = client.getClientData();
		ServerCommandLeave cmd = new ServerCommandLeave("Bob", ClientMode.SPECTATOR, Collections.emptyList());

		assertTrue(handler.handleNetCommand(cmd, ClientCommandHandlerMode.PLAYING));
		assertTrue(handler.handleNetCommand(cmd, ClientCommandHandlerMode.REPLAYING));

		verify(clientData, never()).setTurnTimerStopped(true);
	}

	@Test
	void handleNetCommandUpdatesSpectatorsAndReportsLeaveWhenNotReplaying() {
		handler = new ClientCommandHandlerLeave(client);
		ClientData clientData = client.getClientData();
		UserInterface userInterface = client.getUserInterface();
		LogComponent log = userInterface.getLog();
		StatusReport statusReport = userInterface.getStatusReport();
		List<String> spectators = Collections.singletonList("s1");
		ServerCommandLeave cmd = new ServerCommandLeave("Bob", ClientMode.SPECTATOR, spectators);

		assertTrue(handler.handleNetCommand(cmd, ClientCommandHandlerMode.PLAYING));

		verify(clientData).setSpectatorCount(1);
		verify(clientData).setSpectators(spectators);
		verify(log).markCommandBegin(cmd.getCommandNr());
		verify(statusReport).reportLeave(cmd);
		verify(log).markCommandEnd(cmd.getCommandNr());
		verify(userInterface).invokeAndWait(org.mockito.ArgumentMatchers.any());
	}

	@Test
	void handleNetCommandSkipsReportingWhenReplaying() {
		handler = new ClientCommandHandlerLeave(client);
		ClientData clientData = client.getClientData();
		UserInterface userInterface = client.getUserInterface();
		StatusReport statusReport = userInterface.getStatusReport();
		List<String> spectators = Collections.singletonList("s1");
		ServerCommandLeave cmd = new ServerCommandLeave("Bob", ClientMode.SPECTATOR, spectators);

		assertTrue(handler.handleNetCommand(cmd, ClientCommandHandlerMode.REPLAYING));

		// Spectator bookkeeping happens regardless of replay mode.
		verify(clientData).setSpectatorCount(1);
		verify(clientData).setSpectators(spectators);
		// Reporting/side-bar refresh only happens when not replaying.
		verify(statusReport, never()).reportLeave(cmd);
	}

	// Rust `handle_net_command_is_a_no_op_for_a_mismatched_command_type` SKIPPED: Java casts
	// `(ServerCommandLeave) pNetCommand` unconditionally; a wrong command type throws
	// ClassCastException instead of no-op'ing.
}
