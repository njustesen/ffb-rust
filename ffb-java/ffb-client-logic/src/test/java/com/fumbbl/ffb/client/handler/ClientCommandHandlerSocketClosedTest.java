package com.fumbbl.ffb.client.handler;

import com.fumbbl.ffb.client.FantasyFootballClient;
import com.fumbbl.ffb.client.UserInterface;
import com.fumbbl.ffb.net.NetCommandId;
import com.fumbbl.ffb.net.commands.ServerCommandSound;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.Answers;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.Mockito.verify;

/**
 * Mirrors Rust {@code client_command_handler_socket_closed.rs} tests.
 */
@ExtendWith(MockitoExtension.class)
class ClientCommandHandlerSocketClosedTest {

	@Mock(answer = Answers.RETURNS_DEEP_STUBS)
	private FantasyFootballClient client;

	private ClientCommandHandlerSocketClosed handler;

	@Test
	void getIdReturnsInternalServerSocketClosed() {
		handler = new ClientCommandHandlerSocketClosed(client);
		assertEquals(NetCommandId.INTERNAL_SERVER_SOCKET_CLOSED, handler.getId());
	}

	@Test
	void handleNetCommandAlwaysReturnsTrueAndNotifiesUi() {
		handler = new ClientCommandHandlerSocketClosed(client);
		UserInterface userInterface = client.getUserInterface();

		// Java ignores the command entirely (no cast), so any command type is safe here.
		ServerCommandSound cmd = new ServerCommandSound();

		assertTrue(handler.handleNetCommand(cmd, ClientCommandHandlerMode.PLAYING));

		verify(userInterface).socketClosed();
		verify(client).logDebug("Connection closed by server.");
	}

	@Test
	void handleNetCommandReturnsTrueAcrossAllModes() {
		handler = new ClientCommandHandlerSocketClosed(client);
		ServerCommandSound cmd = new ServerCommandSound();

		assertTrue(handler.handleNetCommand(cmd, ClientCommandHandlerMode.REPLAYING));
		assertTrue(handler.handleNetCommand(cmd, ClientCommandHandlerMode.INITIALIZING));
		assertTrue(handler.handleNetCommand(cmd, ClientCommandHandlerMode.QUEUING));
	}
}
