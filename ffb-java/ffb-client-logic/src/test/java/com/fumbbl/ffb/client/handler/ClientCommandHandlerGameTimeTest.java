package com.fumbbl.ffb.client.handler;

import com.fumbbl.ffb.client.FantasyFootballClient;
import com.fumbbl.ffb.client.GameTitle;
import com.fumbbl.ffb.client.ui.GameTitleUpdateTask;
import com.fumbbl.ffb.net.commands.ServerCommandGameTime;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.ArgumentCaptor;
import org.mockito.Answers;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;
import org.mockito.junit.jupiter.MockitoSettings;
import org.mockito.quality.Strictness;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.Mockito.verify;

/**
 * 1:1 port of the Rust {@code client_command_handler_game_time.rs} test module.
 */
@ExtendWith(MockitoExtension.class)
@MockitoSettings(strictness = Strictness.LENIENT)
class ClientCommandHandlerGameTimeTest {

	@Mock(answer = Answers.RETURNS_DEEP_STUBS)
	private FantasyFootballClient client;

	private ClientCommandHandlerGameTime handler;

	@Test
	void handleNetCommandRecordsGameAndTurnTime() {
		handler = new ClientCommandHandlerGameTime(client);
		ServerCommandGameTime cmd = new ServerCommandGameTime(60_000, 30_000);

		assertTrue(handler.handleNetCommand(cmd, ClientCommandHandlerMode.PLAYING));

		ArgumentCaptor<Runnable> taskCaptor = ArgumentCaptor.forClass(Runnable.class);
		verify(client.getUserInterface()).invokeLater(taskCaptor.capture());
		Runnable task = taskCaptor.getValue();
		assertTrue(task instanceof GameTitleUpdateTask);

		// Running the captured task drives GameTitleUpdateTask.run(), which calls
		// userInterface.getGameTitle().update(gameTitle) with the GameTitle built by
		// the handler -- this is the Java-observable equivalent of Rust's `last_update`.
		task.run();

		ArgumentCaptor<GameTitle> titleCaptor = ArgumentCaptor.forClass(GameTitle.class);
		verify(client.getUserInterface().getGameTitle()).update(titleCaptor.capture());
		assertEquals(60_000, titleCaptor.getValue().getGameTime());
		assertEquals(30_000, titleCaptor.getValue().getTurnTime());
	}

	// Rust: handle_net_command_ignores_mismatched_command_type
	// SKIPPED: Java casts unconditionally; wrong type throws CCE, not a no-op.

	@Test
	void handleNetCommandReturnsTrueAcrossAllModes() {
		handler = new ClientCommandHandlerGameTime(client);
		ServerCommandGameTime cmd = new ServerCommandGameTime(1, 2);

		assertTrue(handler.handleNetCommand(cmd, ClientCommandHandlerMode.REPLAYING));
		assertTrue(handler.handleNetCommand(cmd, ClientCommandHandlerMode.QUEUING));
	}
}
