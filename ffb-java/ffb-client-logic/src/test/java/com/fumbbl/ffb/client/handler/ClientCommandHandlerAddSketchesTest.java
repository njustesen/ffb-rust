package com.fumbbl.ffb.client.handler;

import com.fumbbl.ffb.client.FantasyFootballClient;
import com.fumbbl.ffb.model.sketch.Sketch;
import com.fumbbl.ffb.net.commands.ServerCommandAddSketches;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.Answers;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;

import java.util.Collections;
import java.util.List;

import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.ArgumentMatchers.eq;
import static org.mockito.BDDMockito.given;
import static org.mockito.Mockito.verify;

@ExtendWith(MockitoExtension.class)
class ClientCommandHandlerAddSketchesTest {

	@Mock(answer = Answers.RETURNS_DEEP_STUBS)
	private FantasyFootballClient client;

	private ClientCommandHandlerAddSketches handler;

	@Test
	void handleNetCommandReturnsTrueWithSketches() {
		given(client.getUserInterface().getSketchManager().getAllSketches()).willReturn(Collections.emptyList());
		handler = new ClientCommandHandlerAddSketches(client);
		Sketch sketch = new Sketch(0);
		ServerCommandAddSketches cmd = new ServerCommandAddSketches("Alice", List.of(sketch));

		assertTrue(handler.handleNetCommand(cmd, ClientCommandHandlerMode.PLAYING));
	}

	@Test
	void handleNetCommandReturnsTrueWithNoSketches() {
		given(client.getUserInterface().getSketchManager().getAllSketches()).willReturn(Collections.emptyList());
		handler = new ClientCommandHandlerAddSketches(client);
		ServerCommandAddSketches cmd = new ServerCommandAddSketches("Bob", Collections.emptyList());

		assertTrue(handler.handleNetCommand(cmd, ClientCommandHandlerMode.PLAYING));
	}

	@Test
	void updateSketchManagerIteratesAllSketchesWithoutPanicking() {
		handler = new ClientCommandHandlerAddSketches(client);
		Sketch sketch1 = new Sketch(0);
		Sketch sketch2 = new Sketch(0);
		ServerCommandAddSketches cmd = new ServerCommandAddSketches("Carol", List.of(sketch1, sketch2));

		handler.updateSketchManager(cmd);

		verify(client.getUserInterface().getSketchManager()).add(eq("Carol"), eq(sketch1));
		verify(client.getUserInterface().getSketchManager()).add(eq("Carol"), eq(sketch2));
	}
}
