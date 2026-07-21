package com.fumbbl.ffb.client.handler;

import com.fumbbl.ffb.client.FantasyFootballClient;
import com.fumbbl.ffb.net.commands.ServerCommandSketchSetColor;
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
class ClientCommandHandlerSketchSetColorTest {

	@Mock(answer = Answers.RETURNS_DEEP_STUBS)
	private FantasyFootballClient client;

	private ClientCommandHandlerSketchSetColor handler;

	@Test
	void handleNetCommandReturnsTrueWithMultipleIds() {
		given(client.getUserInterface().getSketchManager().getAllSketches()).willReturn(Collections.emptyList());
		handler = new ClientCommandHandlerSketchSetColor(client);
		ServerCommandSketchSetColor cmd = new ServerCommandSketchSetColor("Bob", List.of("s1", "s2"), 0xFF0000);

		assertTrue(handler.handleNetCommand(cmd, ClientCommandHandlerMode.PLAYING));
	}

	@Test
	void handleNetCommandReturnsTrueWithNoIds() {
		given(client.getUserInterface().getSketchManager().getAllSketches()).willReturn(Collections.emptyList());
		handler = new ClientCommandHandlerSketchSetColor(client);
		ServerCommandSketchSetColor cmd = new ServerCommandSketchSetColor("Bob", Collections.emptyList(), 0);

		assertTrue(handler.handleNetCommand(cmd, ClientCommandHandlerMode.PLAYING));
	}

	@Test
	void updateSketchManagerSetsColorForEachId() {
		handler = new ClientCommandHandlerSketchSetColor(client);
		ServerCommandSketchSetColor cmd = new ServerCommandSketchSetColor("Carol", List.of("a"), 42);

		handler.updateSketchManager(cmd);

		verify(client.getUserInterface().getSketchManager()).setColor("Carol", "a", 42);
	}
}
