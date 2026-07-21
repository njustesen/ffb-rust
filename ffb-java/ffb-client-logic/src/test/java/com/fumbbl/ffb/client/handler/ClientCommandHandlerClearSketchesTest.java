package com.fumbbl.ffb.client.handler;

import com.fumbbl.ffb.client.FantasyFootballClient;
import com.fumbbl.ffb.net.commands.ServerCommandClearSketches;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.Answers;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;

import java.util.Collections;

import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.BDDMockito.given;
import static org.mockito.Mockito.verify;

@ExtendWith(MockitoExtension.class)
class ClientCommandHandlerClearSketchesTest {

	@Mock(answer = Answers.RETURNS_DEEP_STUBS)
	private FantasyFootballClient client;

	private ClientCommandHandlerClearSketches handler;

	@Test
	void handleNetCommandReturnsTrueForMatchingCommand() {
		given(client.getUserInterface().getSketchManager().getAllSketches()).willReturn(Collections.emptyList());
		handler = new ClientCommandHandlerClearSketches(client);
		ServerCommandClearSketches cmd = new ServerCommandClearSketches();

		assertTrue(handler.handleNetCommand(cmd, ClientCommandHandlerMode.PLAYING));
	}

	@Test
	void handleNetCommandReturnsTrueAcrossModes() {
		given(client.getUserInterface().getSketchManager().getAllSketches()).willReturn(Collections.emptyList());
		handler = new ClientCommandHandlerClearSketches(client);
		ServerCommandClearSketches cmd = new ServerCommandClearSketches();

		assertTrue(handler.handleNetCommand(cmd, ClientCommandHandlerMode.REPLAYING));
		assertTrue(handler.handleNetCommand(cmd, ClientCommandHandlerMode.INITIALIZING));
	}

	@Test
	void updateSketchManagerClearsAllSketches() {
		handler = new ClientCommandHandlerClearSketches(client);
		ServerCommandClearSketches cmd = new ServerCommandClearSketches();

		handler.updateSketchManager(cmd);

		verify(client.getUserInterface().getSketchManager()).clearAll();
	}

	// SKIP: Rust "update_sketch_manager_ignores_mismatched_command_type" test —
	// Java's updateSketchManager(ServerCommandClearSketches) is statically typed
	// (no runtime variant match); a wrong-type argument cannot compile, and the
	// dispatch-time cast in the abstract base throws ClassCastException rather
	// than no-op'ing. Not faithfully reproducible in Java.
}
