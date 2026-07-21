package com.fumbbl.ffb.client.handler;

import com.fumbbl.ffb.FieldCoordinate;
import com.fumbbl.ffb.client.FantasyFootballClient;
import com.fumbbl.ffb.net.commands.ServerCommandSketchAddCoordinate;
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
class ClientCommandHandlerSketchAddCoordinateTest {

	@Mock(answer = Answers.RETURNS_DEEP_STUBS)
	private FantasyFootballClient client;

	private ClientCommandHandlerSketchAddCoordinate handler;

	@Test
	void handleNetCommandReturnsTrueForMatchingCommand() {
		given(client.getUserInterface().getSketchManager().getAllSketches()).willReturn(Collections.emptyList());
		handler = new ClientCommandHandlerSketchAddCoordinate(client);
		ServerCommandSketchAddCoordinate cmd = new ServerCommandSketchAddCoordinate("Alice", "sk1", new FieldCoordinate(5, 3));

		assertTrue(handler.handleNetCommand(cmd, ClientCommandHandlerMode.PLAYING));
	}

	@Test
	void updateSketchManagerAddsCoordinateForCoachSketchIdAndCoordinate() {
		handler = new ClientCommandHandlerSketchAddCoordinate(client);
		FieldCoordinate coordinate = new FieldCoordinate(1, 2);
		ServerCommandSketchAddCoordinate cmd = new ServerCommandSketchAddCoordinate("Bob", "sk2", coordinate);

		handler.updateSketchManager(cmd);

		verify(client.getUserInterface().getSketchManager()).add("Bob", "sk2", coordinate);
	}

	@Test
	void handleNetCommandTrueAcrossAllModes() {
		given(client.getUserInterface().getSketchManager().getAllSketches()).willReturn(Collections.emptyList());
		handler = new ClientCommandHandlerSketchAddCoordinate(client);
		ServerCommandSketchAddCoordinate cmd = new ServerCommandSketchAddCoordinate("Alice", "sk1", new FieldCoordinate(0, 0));

		assertTrue(handler.handleNetCommand(cmd, ClientCommandHandlerMode.REPLAYING));
		assertTrue(handler.handleNetCommand(cmd, ClientCommandHandlerMode.QUEUING));
	}
}
