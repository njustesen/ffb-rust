package com.fumbbl.ffb.client.handler;

import com.fumbbl.ffb.client.FantasyFootballClient;
import com.fumbbl.ffb.client.TextStyle;
import com.fumbbl.ffb.client.ui.ChatComponent;
import com.fumbbl.ffb.model.Game;
import com.fumbbl.ffb.model.Team;
import com.fumbbl.ffb.net.NetCommandId;
import com.fumbbl.ffb.net.commands.ServerCommandTalk;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.Answers;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;
import org.mockito.junit.jupiter.MockitoSettings;
import org.mockito.quality.Strictness;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.BDDMockito.given;
import static org.mockito.Mockito.verify;

/**
 * Mirrors Rust {@code client_command_handler_talk.rs} tests. Rust's {@code talk_style}/
 * {@code status_prefix} helpers were invented only because Rust lacked a live Game/UI; Java
 * computes the same style/prefix inline against a real {@code Game} and appends via
 * {@link ChatComponent#parseAndAppend}, so the port drives the same input scenarios through
 * mocks and asserts on that UI seam instead of a standalone helper.
 */
@ExtendWith(MockitoExtension.class)
@MockitoSettings(strictness = Strictness.LENIENT)
class ClientCommandHandlerTalkTest {

	@Mock(answer = Answers.RETURNS_DEEP_STUBS)
	private FantasyFootballClient client;

	@Mock
	private Team teamHome;

	@Mock
	private Team teamAway;

	private ClientCommandHandlerTalk handler;

	private Game game;
	private ChatComponent chat;

	private void setUp() {
		handler = new ClientCommandHandlerTalk(client);
		game = client.getGame();
		chat = client.getUserInterface().getChat();
		given(game.getTeamHome()).willReturn(teamHome);
		given(game.getTeamAway()).willReturn(teamAway);
	}

	@Test
	void getIdReturnsServerTalk() {
		handler = new ClientCommandHandlerTalk(client);
		assertEquals(NetCommandId.SERVER_TALK, handler.getId());
	}

	@Test
	void talkFromHomeCoachUsesHomeStyle() {
		setUp();
		given(teamHome.getCoach()).willReturn("Alice");
		given(teamAway.getCoach()).willReturn("Bob");

		ServerCommandTalk cmd = new ServerCommandTalk("Alice", "hi", ServerCommandTalk.Mode.REGULAR);

		assertTrue(handler.handleNetCommand(cmd, ClientCommandHandlerMode.PLAYING));

		verify(chat).parseAndAppend(TextStyle.HOME, TextStyle.HOME_BOLD, "<Alice> ", "hi");
	}

	@Test
	void talkFromAwayCoachUsesAwayStyle() {
		setUp();
		given(teamHome.getCoach()).willReturn("Alice");
		given(teamAway.getCoach()).willReturn("Bob");

		ServerCommandTalk cmd = new ServerCommandTalk("Bob", "hi", ServerCommandTalk.Mode.REGULAR);

		assertTrue(handler.handleNetCommand(cmd, ClientCommandHandlerMode.PLAYING));

		verify(chat).parseAndAppend(TextStyle.AWAY, TextStyle.AWAY_BOLD, "<Bob> ", "hi");
	}

	@Test
	void talkFromStaffModeUsesAdminStyle() {
		setUp();
		given(teamHome.getCoach()).willReturn("Alice");
		given(teamAway.getCoach()).willReturn("Bob");

		ServerCommandTalk cmd = new ServerCommandTalk("Admin", "hi", ServerCommandTalk.Mode.STAFF);

		assertTrue(handler.handleNetCommand(cmd, ClientCommandHandlerMode.PLAYING));

		verify(chat).parseAndAppend(TextStyle.ADMIN, TextStyle.ADMIN_BOLD, "<Staff Admin> ", "hi");
	}

	@Test
	void talkFromDevModeUsesDevStyle() {
		setUp();
		given(teamHome.getCoach()).willReturn("Alice");
		given(teamAway.getCoach()).willReturn("Bob");

		ServerCommandTalk cmd = new ServerCommandTalk("Dev", "hi", ServerCommandTalk.Mode.DEV);

		assertTrue(handler.handleNetCommand(cmd, ClientCommandHandlerMode.PLAYING));

		verify(chat).parseAndAppend(TextStyle.DEV, TextStyle.DEV_BOLD, "<Dev Dev> ", "hi");
	}

	@Test
	void talkFromOtherRegularCoachDefaultsToSpectatorStyle() {
		setUp();
		given(teamHome.getCoach()).willReturn("Alice");
		given(teamAway.getCoach()).willReturn("Bob");

		ServerCommandTalk cmd = new ServerCommandTalk("Watcher", "hi", ServerCommandTalk.Mode.REGULAR);

		assertTrue(handler.handleNetCommand(cmd, ClientCommandHandlerMode.PLAYING));

		verify(chat).parseAndAppend(TextStyle.SPECTATOR, TextStyle.SPECTATOR_BOLD, "<Watcher> ", "hi");
	}

	@Test
	void handleNetCommandReturnsTrueForMatchingCommand() {
		setUp();
		given(teamHome.getCoach()).willReturn("Alice");
		given(teamAway.getCoach()).willReturn("Bob");

		ServerCommandTalk cmd = new ServerCommandTalk("Alice", new String[]{"hi"});

		assertTrue(handler.handleNetCommand(cmd, ClientCommandHandlerMode.PLAYING));
	}

	// Rust `handle_net_command_is_a_no_op_for_a_mismatched_command_type` SKIPPED: Java casts
	// `(ServerCommandTalk) pNetCommand` unconditionally; a wrong command type throws
	// ClassCastException instead of no-op'ing.
}
