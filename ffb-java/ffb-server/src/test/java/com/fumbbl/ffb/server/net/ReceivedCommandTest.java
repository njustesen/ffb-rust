package com.fumbbl.ffb.server.net;

import com.fumbbl.ffb.net.NetCommand;
import com.fumbbl.ffb.net.commands.ClientCommand;
import org.eclipse.jetty.websocket.api.Session;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertSame;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.Mockito.mock;
import static org.mockito.Mockito.when;

/**
 * Mirrors the Rust {@code net::received_command} tests. Rust stores plain
 * internal/client flags; Java derives them from the wrapped {@link NetCommand}.
 */
class ReceivedCommandTest {

	@Test
	void isInternalCommand() {
		NetCommand command = mock(NetCommand.class);
		when(command.isInternal()).thenReturn(true);
		ReceivedCommand rc = new ReceivedCommand(command, mock(Session.class));
		assertTrue(rc.isInternalCommand());
	}

	@Test
	void isClientCommand() {
		ClientCommand command = mock(ClientCommand.class);
		ReceivedCommand rc = new ReceivedCommand(command, mock(Session.class));
		assertTrue(rc.isClientCommand());
		assertFalse(rc.isInternalCommand());
	}

	@Test
	void getSession() {
		Session session = mock(Session.class);
		ReceivedCommand rc = new ReceivedCommand(mock(ClientCommand.class), session);
		assertSame(session, rc.getSession());
	}
}
