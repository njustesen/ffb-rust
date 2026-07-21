package com.fumbbl.ffb.client.handler;

import com.fumbbl.ffb.client.FantasyFootballClient;
import com.fumbbl.ffb.net.NetCommandId;
import com.fumbbl.ffb.net.commands.ServerCommandClearSketches;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.Answers;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;

import java.util.Collections;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.BDDMockito.given;

@ExtendWith(MockitoExtension.class)
class AbstractClientCommandHandlerSketchTest {

	@Mock(answer = Answers.RETURNS_DEEP_STUBS)
	private FantasyFootballClient client;

	/**
	 * Minimal concrete subclass mirroring the Rust test's Recorder struct:
	 * records how many times updateSketchManager was invoked so the
	 * final handleNetCommand behavior can be observed independently of any
	 * particular concrete handler.
	 */
	private static class RecorderHandler extends AbstractClientCommandHandlerSketch<ServerCommandClearSketches> {
		int calls;

		RecorderHandler(FantasyFootballClient client) {
			super(client);
		}

		@Override
		public NetCommandId getId() {
			return NetCommandId.SERVER_CLEAR_SKETCHES;
		}

		@Override
		protected void updateSketchManager(ServerCommandClearSketches command) {
			calls++;
		}
	}

	@Test
	void handleNetCommandCallsUpdateSketchManager() {
		given(client.getUserInterface().getSketchManager().getAllSketches()).willReturn(Collections.emptyList());
		RecorderHandler handler = new RecorderHandler(client);
		ServerCommandClearSketches cmd = new ServerCommandClearSketches();

		boolean handled = handler.handleNetCommand(cmd, ClientCommandHandlerMode.PLAYING);

		assertEquals(1, handler.calls);
		assertTrue(handled);
	}

	@Test
	void handleNetCommandAlwaysReturnsTrue() {
		given(client.getUserInterface().getSketchManager().getAllSketches()).willReturn(Collections.emptyList());
		RecorderHandler handler = new RecorderHandler(client);
		ServerCommandClearSketches cmd = new ServerCommandClearSketches();

		assertTrue(handler.handleNetCommand(cmd, ClientCommandHandlerMode.REPLAYING));
		assertTrue(handler.handleNetCommand(cmd, ClientCommandHandlerMode.QUEUING));
	}
}
