package com.fumbbl.ffb.client.handler;

import com.fumbbl.ffb.client.FantasyFootballClient;
import com.fumbbl.ffb.net.commands.ServerCommandSketchSetLabel;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.Answers;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;

import java.util.Collections;
import java.util.List;

import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.BDDMockito.given;
import static org.mockito.Mockito.verify;

@ExtendWith(MockitoExtension.class)
class ClientCommandHandlerSketchSetLabelTest {

	@Mock(answer = Answers.RETURNS_DEEP_STUBS)
	private FantasyFootballClient client;

	private ClientCommandHandlerSketchSetLabel handler;

	@Test
	void handleNetCommandReturnsTrueWithMultipleIds() {
		given(client.getUserInterface().getSketchManager().getAllSketches()).willReturn(Collections.emptyList());
		handler = new ClientCommandHandlerSketchSetLabel(client);
		ServerCommandSketchSetLabel cmd = new ServerCommandSketchSetLabel("Carol", List.of("s1"), "Arrow");

		assertTrue(handler.handleNetCommand(cmd, ClientCommandHandlerMode.PLAYING));
	}

	@Test
	void handleNetCommandReturnsTrueWithNoIds() {
		given(client.getUserInterface().getSketchManager().getAllSketches()).willReturn(Collections.emptyList());
		handler = new ClientCommandHandlerSketchSetLabel(client);
		ServerCommandSketchSetLabel cmd = new ServerCommandSketchSetLabel("Carol", Collections.emptyList(), "");

		assertTrue(handler.handleNetCommand(cmd, ClientCommandHandlerMode.PLAYING));
	}

	@Test
	void updateSketchManagerSetsLabelForEachId() {
		handler = new ClientCommandHandlerSketchSetLabel(client);
		ServerCommandSketchSetLabel cmd = new ServerCommandSketchSetLabel("Dave", List.of("s1", "s2"), "Circle");

		handler.updateSketchManager(cmd);

		verify(client.getUserInterface().getSketchManager()).setLabel("Dave", "s1", "Circle");
		verify(client.getUserInterface().getSketchManager()).setLabel("Dave", "s2", "Circle");
	}
}
