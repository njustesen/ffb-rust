package com.fumbbl.ffb.client.handler;

import com.fumbbl.ffb.PlayerState;
import com.fumbbl.ffb.SendToBoxReason;
import com.fumbbl.ffb.client.FantasyFootballClient;
import com.fumbbl.ffb.model.FieldModel;
import com.fumbbl.ffb.model.Game;
import com.fumbbl.ffb.model.GameResult;
import com.fumbbl.ffb.model.PlayerResult;
import com.fumbbl.ffb.model.Roster;
import com.fumbbl.ffb.model.RosterPlayer;
import com.fumbbl.ffb.model.Team;
import com.fumbbl.ffb.net.commands.ServerCommandAddPlayer;
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
 * client_command_handler_add_player.rs. The Rust tests exercise a free {@code apply_to_game(&mut Game)}
 * function; Java has no such seam, so these tests drive the same scenarios through
 * {@link ClientCommandHandlerAddPlayer#handleNetCommand} with a mocked client/game graph and verify
 * the Java-observable side effects (team/field-model/player-result mutations) instead of asserting on
 * plain struct state.
 */
@ExtendWith(MockitoExtension.class)
@MockitoSettings(strictness = Strictness.LENIENT)
class ClientCommandHandlerAddPlayerTest {

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
	private Roster roster;

	@Mock
	private RosterPlayer player;

	@Mock
	private RosterPlayer existingPlayer;

	private ClientCommandHandlerAddPlayer handler;

	@BeforeEach
	void setUp() {
		handler = new ClientCommandHandlerAddPlayer(client);

		given(client.getGame()).willReturn(game);
		given(game.getTeamHome()).willReturn(teamHome);
		given(game.getTeamAway()).willReturn(teamAway);
		given(game.getFieldModel()).willReturn(fieldModel);
		given(game.getGameResult()).willReturn(gameResult);
		given(teamHome.getId()).willReturn("home");
		given(teamAway.getId()).willReturn("away");
		given(teamHome.getRoster()).willReturn(roster);
		given(teamAway.getRoster()).willReturn(roster);
		given(player.getId()).willReturn("p1");
		// UtilBox.putPlayerIntoBox reads fieldModel.getPlayerState(player).getBase(); give it a real state.
		given(fieldModel.getPlayerState(player)).willReturn(new com.fumbbl.ffb.PlayerState(com.fumbbl.ffb.PlayerState.STANDING));
		// Handler reads game.getGameResult().getPlayerResult(player) then sets fields on it.
		given(gameResult.getPlayerResult(player)).willReturn(org.mockito.Mockito.mock(PlayerResult.class));
	}

	@Test
	void addsNewPlayerToCorrectTeam() {
		given(teamHome.getPlayerById("p1")).willReturn(null);
		ServerCommandAddPlayer command = new ServerCommandAddPlayer("home", player, new PlayerState(PlayerState.STANDING), null);

		assertTrue(handler.handleNetCommand(command, ClientCommandHandlerMode.PLAYING));

		verify(teamHome).addPlayer(player);
		verify(teamAway, never()).addPlayer(any());
	}

	@Test
	void replacesExistingPlayerInPlace() {
		given(teamHome.getPlayerById("p1")).willReturn((com.fumbbl.ffb.model.Player) existingPlayer);
		ServerCommandAddPlayer command = new ServerCommandAddPlayer("home", player, new PlayerState(PlayerState.STANDING), null);

		assertTrue(handler.handleNetCommand(command, ClientCommandHandlerMode.PLAYING));

		verify(existingPlayer).init(player, game.getRules());
		verify(teamHome, never()).addPlayer(any());
	}

	@Test
	void setsFieldModelPlayerState() {
		given(teamHome.getPlayerById("p1")).willReturn(null);
		PlayerState state = new PlayerState(5);
		ServerCommandAddPlayer command = new ServerCommandAddPlayer("home", player, state, null);

		assertTrue(handler.handleNetCommand(command, ClientCommandHandlerMode.PLAYING));

		verify(fieldModel).setPlayerState(player, state);
	}

	@Test
	void setsSendToBoxFieldsOnPlayerResult() {
		given(teamHome.getPlayerById("p1")).willReturn(null);

		PlayerResult inputResult = org.mockito.Mockito.mock(PlayerResult.class);
		given(inputResult.getSendToBoxReason()).willReturn(SendToBoxReason.MNG);
		given(inputResult.getSendToBoxTurn()).willReturn(3);
		given(inputResult.getSendToBoxHalf()).willReturn(0);

		PlayerResult actualResult = org.mockito.Mockito.mock(PlayerResult.class);
		given(gameResult.getPlayerResult(player)).willReturn(actualResult);

		ServerCommandAddPlayer command = new ServerCommandAddPlayer("home", player, new PlayerState(PlayerState.STANDING), inputResult);

		assertTrue(handler.handleNetCommand(command, ClientCommandHandlerMode.PLAYING));

		verify(actualResult).setSendToBoxReason(SendToBoxReason.MNG);
		verify(actualResult).setSendToBoxTurn(3);
		verify(actualResult).setSendToBoxHalf(0);
	}

	// SKIPPED: handle_net_command_is_a_no_op_for_a_mismatched_command_type
	// Java casts unconditionally; wrong type throws CCE, not a no-op.
}
