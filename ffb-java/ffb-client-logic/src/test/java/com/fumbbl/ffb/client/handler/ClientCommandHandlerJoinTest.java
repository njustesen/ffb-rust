package com.fumbbl.ffb.client.handler;

import com.fumbbl.ffb.ClientMode;
import com.fumbbl.ffb.client.ClientData;
import com.fumbbl.ffb.client.ClientParameters;
import com.fumbbl.ffb.client.FantasyFootballClient;
import com.fumbbl.ffb.client.GameTitle;
import com.fumbbl.ffb.client.StatusReport;
import com.fumbbl.ffb.client.UserInterface;
import com.fumbbl.ffb.client.ui.LogComponent;
import com.fumbbl.ffb.net.NetCommandId;
import com.fumbbl.ffb.net.commands.ServerCommandJoin;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.Answers;
import org.mockito.ArgumentCaptor;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;
import org.mockito.junit.jupiter.MockitoSettings;
import org.mockito.quality.Strictness;

import java.util.Collections;
import java.util.List;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.ArgumentMatchers.any;
import static org.mockito.BDDMockito.given;
import static org.mockito.Mockito.never;
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.verifyNoInteractions;

/**
 * Mirrors Rust {@code client_command_handler_join.rs} tests. Rust's
 * {@code home_away_coaches}/{@code should_resume_turn_timer}/{@code should_report_join} were
 * invented purely because {@code ClientParameters}/{@code Game.getTeamHome()} etc. weren't
 * reachable there. Java drives the same logic inline and directly reachable through mocks, so
 * the port asserts on the Java-observable seams (a real {@code GameTitle} captured off the
 * {@code invokeLater} task, {@link ClientData}, {@link StatusReport}) instead of standalone
 * helpers.
 */
@ExtendWith(MockitoExtension.class)
@MockitoSettings(strictness = Strictness.LENIENT)
class ClientCommandHandlerJoinTest {

	@Mock(answer = Answers.RETURNS_DEEP_STUBS)
	private FantasyFootballClient client;

	private ClientCommandHandlerJoin handler;

	@Test
	void getIdReturnsServerJoin() {
		handler = new ClientCommandHandlerJoin(client);
		assertEquals(NetCommandId.SERVER_JOIN, handler.getId());
	}

	@Test
	void handleNetCommandShortCircuitsWhenQueuing() {
		handler = new ClientCommandHandlerJoin(client);
		ServerCommandJoin cmd = new ServerCommandJoin("Bob", ClientMode.PLAYER, new String[0], Collections.emptyList(), "");

		assertTrue(handler.handleNetCommand(cmd, ClientCommandHandlerMode.QUEUING));

		verifyNoInteractions(client);
	}

	@Test
	void handleNetCommandResumesTurnTimerForPlayerMode() {
		handler = new ClientCommandHandlerJoin(client);
		ClientData clientData = client.getClientData();
		ServerCommandJoin cmd = new ServerCommandJoin("Bob", ClientMode.PLAYER, new String[0], Collections.emptyList(), "");

		assertTrue(handler.handleNetCommand(cmd, ClientCommandHandlerMode.PLAYING));

		verify(clientData).setTurnTimerStopped(false);
	}

	@Test
	void handleNetCommandDoesNotResumeTurnTimerForSpectatorMode() {
		handler = new ClientCommandHandlerJoin(client);
		ClientData clientData = client.getClientData();
		ServerCommandJoin cmd = new ServerCommandJoin("Bob", ClientMode.SPECTATOR, new String[0], Collections.emptyList(), "");

		assertTrue(handler.handleNetCommand(cmd, ClientCommandHandlerMode.PLAYING));

		verify(clientData, never()).setTurnTimerStopped(false);
	}

	@Test
	void handleNetCommandSwapsHomeAwayWhenSecondPlayerIsTheLocalCoach() {
		handler = new ClientCommandHandlerJoin(client);
		UserInterface userInterface = client.getUserInterface();
		ClientParameters parameters = client.getParameters();
		given(parameters.getCoach()).willReturn("Bob");
		given(client.getMode()).willReturn(ClientMode.PLAYER);
		ServerCommandJoin cmd = new ServerCommandJoin("Bob", ClientMode.PLAYER,
			new String[]{"Alice", "Bob"}, Collections.emptyList(), "");

		assertTrue(handler.handleNetCommand(cmd, ClientCommandHandlerMode.PLAYING));

		ArgumentCaptor<Runnable> taskCaptor = ArgumentCaptor.forClass(Runnable.class);
		verify(userInterface).invokeLater(taskCaptor.capture());
		taskCaptor.getValue().run();

		ArgumentCaptor<GameTitle> titleCaptor = ArgumentCaptor.forClass(GameTitle.class);
		verify(userInterface.getGameTitle()).update(titleCaptor.capture());
		assertEquals("Bob", titleCaptor.getValue().getHomeCoach());
		assertEquals("Alice", titleCaptor.getValue().getAwayCoach());
		assertEquals(ClientMode.PLAYER, titleCaptor.getValue().getClientMode());
	}

	@Test
	void handleNetCommandKeepsHomeAwayOrderOtherwise() {
		handler = new ClientCommandHandlerJoin(client);
		UserInterface userInterface = client.getUserInterface();
		ClientParameters parameters = client.getParameters();
		given(parameters.getCoach()).willReturn("Someone Else");
		ServerCommandJoin cmd = new ServerCommandJoin("Bob", ClientMode.PLAYER,
			new String[]{"Alice", "Bob"}, Collections.emptyList(), "");

		assertTrue(handler.handleNetCommand(cmd, ClientCommandHandlerMode.PLAYING));

		ArgumentCaptor<Runnable> taskCaptor = ArgumentCaptor.forClass(Runnable.class);
		verify(userInterface).invokeLater(taskCaptor.capture());
		taskCaptor.getValue().run();

		ArgumentCaptor<GameTitle> titleCaptor = ArgumentCaptor.forClass(GameTitle.class);
		verify(userInterface.getGameTitle()).update(titleCaptor.capture());
		assertEquals("Alice", titleCaptor.getValue().getHomeCoach());
		assertEquals("Bob", titleCaptor.getValue().getAwayCoach());
	}

	@Test
	void handleNetCommandSkipsGameTitleUpdateWhenFewerThanTwoPlayerNames() {
		handler = new ClientCommandHandlerJoin(client);
		UserInterface userInterface = client.getUserInterface();
		ServerCommandJoin cmd = new ServerCommandJoin("Bob", ClientMode.PLAYER,
			new String[]{"Alice"}, Collections.emptyList(), "");

		assertTrue(handler.handleNetCommand(cmd, ClientCommandHandlerMode.PLAYING));

		verify(userInterface, never()).invokeLater(any());
	}

	@Test
	void handleNetCommandUpdatesSpectatorsAndReportsJoinForOtherCoachWhilePlaying() {
		handler = new ClientCommandHandlerJoin(client);
		ClientData clientData = client.getClientData();
		UserInterface userInterface = client.getUserInterface();
		LogComponent log = userInterface.getLog();
		StatusReport statusReport = userInterface.getStatusReport();
		ClientParameters parameters = client.getParameters();
		given(parameters.getCoach()).willReturn("Alice");
		List<String> spectators = Collections.singletonList("s1");
		ServerCommandJoin cmd = new ServerCommandJoin("Bob", ClientMode.SPECTATOR,
			new String[0], spectators, "");

		assertTrue(handler.handleNetCommand(cmd, ClientCommandHandlerMode.PLAYING));

		verify(clientData).setSpectatorCount(1);
		verify(clientData).setSpectators(spectators);
		verify(log).markCommandBegin(cmd.getCommandNr());
		verify(statusReport).reportJoin(cmd);
		verify(log).markCommandEnd(cmd.getCommandNr());
	}

	@Test
	void handleNetCommandDoesNotReportJoinWhenReplaying() {
		handler = new ClientCommandHandlerJoin(client);
		UserInterface userInterface = client.getUserInterface();
		StatusReport statusReport = userInterface.getStatusReport();
		ClientParameters parameters = client.getParameters();
		given(parameters.getCoach()).willReturn("Alice");
		ServerCommandJoin cmd = new ServerCommandJoin("Bob", ClientMode.SPECTATOR,
			new String[0], Collections.emptyList(), "");

		assertTrue(handler.handleNetCommand(cmd, ClientCommandHandlerMode.REPLAYING));

		verify(statusReport, never()).reportJoin(cmd);
	}

	@Test
	void handleNetCommandDoesNotReportJoinForSelf() {
		handler = new ClientCommandHandlerJoin(client);
		UserInterface userInterface = client.getUserInterface();
		StatusReport statusReport = userInterface.getStatusReport();
		ClientParameters parameters = client.getParameters();
		given(parameters.getCoach()).willReturn("Bob");
		ServerCommandJoin cmd = new ServerCommandJoin("Bob", ClientMode.PLAYER,
			new String[0], Collections.emptyList(), "");

		assertTrue(handler.handleNetCommand(cmd, ClientCommandHandlerMode.PLAYING));

		verify(statusReport, never()).reportJoin(cmd);
	}

	// Rust `handle_net_command_is_a_no_op_for_a_mismatched_command_type` SKIPPED: Java casts
	// `(ServerCommandJoin) pNetCommand` unconditionally; a wrong command type throws
	// ClassCastException instead of no-op'ing.
}
