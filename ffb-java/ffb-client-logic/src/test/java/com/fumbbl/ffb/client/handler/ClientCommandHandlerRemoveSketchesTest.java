package com.fumbbl.ffb.client.handler;

import com.fumbbl.ffb.client.FantasyFootballClient;
import com.fumbbl.ffb.net.commands.ServerCommandRemoveSketches;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.Answers;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;

import java.util.Collections;
import java.util.List;

import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.BDDMockito.given;
import static org.mockito.Mockito.never;
import static org.mockito.Mockito.verify;

@ExtendWith(MockitoExtension.class)
class ClientCommandHandlerRemoveSketchesTest {

	@Mock(answer = Answers.RETURNS_DEEP_STUBS)
	private FantasyFootballClient client;

	private ClientCommandHandlerRemoveSketches handler;

	@Test
	void emptyIdsTakesRemoveAllBranch() {
		given(client.getUserInterface().getSketchManager().getAllSketches()).willReturn(Collections.emptyList());
		handler = new ClientCommandHandlerRemoveSketches(client);
		ServerCommandRemoveSketches cmd = new ServerCommandRemoveSketches("Bob", Collections.emptyList());

		assertTrue(handler.handleNetCommand(cmd, ClientCommandHandlerMode.PLAYING));
		verify(client.getUserInterface().getSketchManager()).removeAll("Bob");
		verify(client.getUserInterface().getSketchManager(), never()).remove("Bob", "id1");
	}

	@Test
	void nonEmptyIdsTakesRemoveIndividualBranch() {
		given(client.getUserInterface().getSketchManager().getAllSketches()).willReturn(Collections.emptyList());
		handler = new ClientCommandHandlerRemoveSketches(client);
		ServerCommandRemoveSketches cmd = new ServerCommandRemoveSketches("Bob", List.of("id1"));

		assertTrue(handler.handleNetCommand(cmd, ClientCommandHandlerMode.PLAYING));
		verify(client.getUserInterface().getSketchManager()).remove("Bob", "id1");
		verify(client.getUserInterface().getSketchManager(), never()).removeAll("Bob");
	}

	@Test
	void nullIdsAlsoTakesRemoveAllBranch() {
		// Java's condition is `getIds() == null || getIds().isEmpty()`; the Rust
		// port only exercises the empty-list case (its ids field is a non-nullable
		// Vec), so this additionally covers the null-ids branch that only Java's
		// nullable List<String> can reach.
		given(client.getUserInterface().getSketchManager().getAllSketches()).willReturn(Collections.emptyList());
		handler = new ClientCommandHandlerRemoveSketches(client);
		ServerCommandRemoveSketches cmd = new ServerCommandRemoveSketches("Bob", null);

		assertTrue(handler.handleNetCommand(cmd, ClientCommandHandlerMode.PLAYING));
		verify(client.getUserInterface().getSketchManager()).removeAll("Bob");
	}

	@Test
	void handleNetCommandReturnsTrueRegardlessOfBranch() {
		given(client.getUserInterface().getSketchManager().getAllSketches()).willReturn(Collections.emptyList());
		handler = new ClientCommandHandlerRemoveSketches(client);
		ServerCommandRemoveSketches empty = new ServerCommandRemoveSketches("A", Collections.emptyList());
		ServerCommandRemoveSketches nonEmpty = new ServerCommandRemoveSketches("A", List.of("x"));

		assertTrue(handler.handleNetCommand(empty, ClientCommandHandlerMode.PLAYING));
		assertTrue(handler.handleNetCommand(nonEmpty, ClientCommandHandlerMode.PLAYING));
	}
}
