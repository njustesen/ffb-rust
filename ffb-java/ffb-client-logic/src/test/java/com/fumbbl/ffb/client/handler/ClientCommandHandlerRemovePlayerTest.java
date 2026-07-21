package com.fumbbl.ffb.client.handler;

import com.fumbbl.ffb.client.FantasyFootballClient;
import com.fumbbl.ffb.model.FieldModel;
import com.fumbbl.ffb.model.Game;
import com.fumbbl.ffb.model.GameResult;
import com.fumbbl.ffb.model.RosterPlayer;
import com.fumbbl.ffb.model.Team;
import com.fumbbl.ffb.model.TeamResult;
import com.fumbbl.ffb.net.commands.ServerCommandRemovePlayer;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.Answers;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;
import org.mockito.junit.jupiter.MockitoSettings;
import org.mockito.quality.Strictness;

import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.ArgumentMatchers.any;
import static org.mockito.BDDMockito.given;
import static org.mockito.Mockito.never;
import static org.mockito.Mockito.verify;

/**
 * Mirrors the Rust {@code apply_to_game}/{@code handle_net_command} unit tests in
 * client_command_handler_remove_player.rs. The Rust tests exercise a free {@code apply_to_game(&mut Game)}
 * function; Java has no such seam, so these tests drive the same scenarios through
 * {@link ClientCommandHandlerRemovePlayer#handleNetCommand} with a mocked client/game graph and verify
 * the Java-observable side effects (team/field-model/team-result mutations) instead of asserting on
 * plain struct state.
 */
@ExtendWith(MockitoExtension.class)
@MockitoSettings(strictness = Strictness.LENIENT)
class ClientCommandHandlerRemovePlayerTest {

	@Mock(answer = Answers.RETURNS_DEEP_STUBS)
	private FantasyFootballClient client;

	@Mock
	private Game game;

	@Mock
	private Team teamHome;

	@Mock
	private Team teamAway;

	@Mock
	private FieldModel fieldModel;

	@Mock
	private GameResult gameResult;

	@Mock
	private TeamResult teamResultHome;

	@Mock
	private TeamResult teamResultAway;

	@Mock
	private RosterPlayer player;

	private ClientCommandHandlerRemovePlayer handler;

	@BeforeEach
	void setUp() {
		handler = new ClientCommandHandlerRemovePlayer(client);

		given(client.getGame()).willReturn(game);
		given(game.getFieldModel()).willReturn(fieldModel);
		given(game.getTeamHome()).willReturn(teamHome);
		given(game.getTeamAway()).willReturn(teamAway);
		given(game.getGameResult()).willReturn(gameResult);
		given(gameResult.getTeamResultHome()).willReturn(teamResultHome);
		given(gameResult.getTeamResultAway()).willReturn(teamResultAway);
	}

	@Test
	void removesPlayerFromHomeTeam() {
		given(game.getPlayerById("p1")).willReturn((com.fumbbl.ffb.model.Player) player);
		given(teamHome.hasPlayer(player)).willReturn(true);
		ServerCommandRemovePlayer command = new ServerCommandRemovePlayer("p1");

		assertTrue(handler.handleNetCommand(command, ClientCommandHandlerMode.PLAYING));

		verify(teamHome).removePlayer(player);
		verify(teamResultHome).removePlayerResult(player);
		verify(teamAway, never()).removePlayer(any());
	}

	@Test
	void clearsFieldModelState() {
		given(game.getPlayerById("p1")).willReturn((com.fumbbl.ffb.model.Player) player);
		ServerCommandRemovePlayer command = new ServerCommandRemovePlayer("p1");

		assertTrue(handler.handleNetCommand(command, ClientCommandHandlerMode.PLAYING));

		verify(fieldModel).remove(player);
		verify(fieldModel).setPlayerState(player, null);
	}

	@Test
	void isANoOpForUnknownPlayer() {
		given(game.getPlayerById("nobody")).willReturn(null);
		ServerCommandRemovePlayer command = new ServerCommandRemovePlayer("nobody");

		assertTrue(handler.handleNetCommand(command, ClientCommandHandlerMode.PLAYING));

		verify(teamHome, never()).removePlayer(any());
		verify(teamAway, never()).removePlayer(any());
	}

	@Test
	void handleNetCommandReturnsTrueForMatchingCommand() {
		given(game.getPlayerById("p1")).willReturn((com.fumbbl.ffb.model.Player) player);
		ServerCommandRemovePlayer command = new ServerCommandRemovePlayer("p1");

		assertTrue(handler.handleNetCommand(command, ClientCommandHandlerMode.PLAYING));
	}

	// SKIPPED: handle_net_command_is_a_no_op_for_a_mismatched_command_type
	// Java casts unconditionally; wrong type throws CCE, not a no-op.
}
