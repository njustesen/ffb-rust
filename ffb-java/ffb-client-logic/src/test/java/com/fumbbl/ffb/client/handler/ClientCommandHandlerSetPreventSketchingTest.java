package com.fumbbl.ffb.client.handler;

import com.fumbbl.ffb.client.FantasyFootballClient;
import com.fumbbl.ffb.client.TextStyle;
import com.fumbbl.ffb.net.commands.ServerCommandSetPreventSketching;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.Answers;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;
import org.mockito.quality.Strictness;
import org.mockito.junit.jupiter.MockitoSettings;

import java.util.Collections;

import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.mockito.BDDMockito.given;
import static org.mockito.Mockito.verify;

@ExtendWith(MockitoExtension.class)
@MockitoSettings(strictness = Strictness.LENIENT)
class ClientCommandHandlerSetPreventSketchingTest {

	@Mock(answer = Answers.RETURNS_DEEP_STUBS)
	private FantasyFootballClient client;

	private ClientCommandHandlerSetPreventSketching handler;

	@Test
	void statusMessageForLocalCoach() {
		given(client.getParameters().getCoach()).willReturn("Alice");
		given(client.getUserInterface().getSketchManager().getAllSketches()).willReturn(Collections.emptyList());
		handler = new ClientCommandHandlerSetPreventSketching(client);
		ServerCommandSetPreventSketching cmd = new ServerCommandSetPreventSketching("Alice", true);

		handler.handleNetCommand(cmd, ClientCommandHandlerMode.PLAYING);

		verify(client.getUserInterface().getChat()).append(TextStyle.SPECTATOR, "You are blocked");
		verify(client.getUserInterface().getSketchManager()).preventedFromSketching("Alice");
	}

	@Test
	void statusMessageForOtherCoach() {
		given(client.getParameters().getCoach()).willReturn("Alice");
		given(client.getUserInterface().getSketchManager().getAllSketches()).willReturn(Collections.emptyList());
		handler = new ClientCommandHandlerSetPreventSketching(client);
		ServerCommandSetPreventSketching cmd = new ServerCommandSetPreventSketching("Bob", false);

		handler.handleNetCommand(cmd, ClientCommandHandlerMode.PLAYING);

		verify(client.getUserInterface().getChat()).append(TextStyle.SPECTATOR, "Coach Bob is unblocked");
		verify(client.getUserInterface().getSketchManager()).allowSketching("Bob");
	}

	@Test
	void handleNetCommandAlwaysReturnsFalseForMatchingCommand() {
		given(client.getParameters().getCoach()).willReturn("Alice");
		given(client.getUserInterface().getSketchManager().getAllSketches()).willReturn(Collections.emptyList());
		handler = new ClientCommandHandlerSetPreventSketching(client);
		ServerCommandSetPreventSketching cmd = new ServerCommandSetPreventSketching("Alice", true);

		assertFalse(handler.handleNetCommand(cmd, ClientCommandHandlerMode.PLAYING));
	}

	// SKIP: Rust "handle_net_command_returns_false_for_a_mismatched_command_type_too" —
	// Java force-casts (ServerCommandSetPreventSketching) pNetCommand unconditionally;
	// a wrong-type NetCommand throws ClassCastException instead of no-op'ing false.
}
