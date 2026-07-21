package com.fumbbl.ffb.client.state.logic;

import com.fumbbl.ffb.ClientStateId;
import com.fumbbl.ffb.FieldCoordinate;
import com.fumbbl.ffb.client.FantasyFootballClient;
import com.fumbbl.ffb.client.net.ClientCommunication;
import com.fumbbl.ffb.model.ActingPlayer;
import com.fumbbl.ffb.model.Player;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.Answers;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;
import org.mockito.junit.jupiter.MockitoSettings;
import org.mockito.quality.Strictness;

import java.util.Arrays;
import java.util.Collections;
import java.util.Optional;

import static org.junit.jupiter.api.Assertions.assertDoesNotThrow;
import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertSame;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.BDDMockito.given;
import static org.mockito.Mockito.verify;

@ExtendWith(MockitoExtension.class)
@MockitoSettings(strictness = Strictness.LENIENT)
class WaitForOpponentLogicModuleTest {

	@Mock(answer = Answers.RETURNS_DEEP_STUBS)
	private FantasyFootballClient client;

	@Mock
	private ActingPlayer actingPlayer;

	@Mock
	private Player<?> player;

	@Mock
	private Player<?> homePlayer;

	@Mock
	private Player<?> awayPlayer;

	@Test
	public void testGetIdReturnsWaitForOpponent() {
		WaitForOpponentLogicModule module = new WaitForOpponentLogicModule(client);
		assertEquals(ClientStateId.WAIT_FOR_OPPONENT, module.getId());
	}

	@Test
	public void testAvailableActionsIsEmpty() {
		assertTrue(new WaitForOpponentLogicModule(client).availableActions().isEmpty());
	}

	@Test
	public void testPerformAvailableActionIsNoOp() {
		WaitForOpponentLogicModule module = new WaitForOpponentLogicModule(client);
		assertDoesNotThrow(() -> module.performAvailableAction(player, ClientAction.MOVE));
	}

	@Test
	public void testActionContextThrows() {
		WaitForOpponentLogicModule module = new WaitForOpponentLogicModule(client);
		UnsupportedOperationException exception = assertThrows(UnsupportedOperationException.class,
			() -> module.actionContext(actingPlayer));
		assertEquals("actionContext for acting player is not supported in waiting context", exception.getMessage());
	}

	@Test
	public void testIllegalProcedureSendsCommand() {
		ClientCommunication communication = client.getCommunication();
		WaitForOpponentLogicModule module = new WaitForOpponentLogicModule(client);
		module.illegalProcedure();
		verify(communication).sendIllegalProcedure();
	}

	@Test
	public void testGetPlayerReturnsEmptyWhenNoPlayersOnSquare() {
		given(client.getGame().getFieldModel().getPlayers(new FieldCoordinate(1, 1))).willReturn(Collections.emptyList());
		WaitForOpponentLogicModule module = new WaitForOpponentLogicModule(client);
		assertFalse(module.getPlayer(new FieldCoordinate(1, 1)).isPresent());
	}

	@Test
	public void testGetPlayerReturnsSingleOccupant() {
		given(client.getGame().getFieldModel().getPlayers(new FieldCoordinate(3, 3)))
			.willReturn(Collections.singletonList(player));
		WaitForOpponentLogicModule module = new WaitForOpponentLogicModule(client);
		Optional<Player<?>> found = module.getPlayer(new FieldCoordinate(3, 3));
		assertTrue(found.isPresent());
		assertSame(player, found.get());
	}

	@Test
	public void testGetPlayerPrefersHomeTeamPlayerAmongMultiple() {
		given(client.getGame().getFieldModel().getPlayers(new FieldCoordinate(4, 4)))
			.willReturn(Arrays.asList(awayPlayer, homePlayer));
		given(client.getGame().getTeamHome().hasPlayer(awayPlayer)).willReturn(false);
		given(client.getGame().getTeamHome().hasPlayer(homePlayer)).willReturn(true);

		WaitForOpponentLogicModule module = new WaitForOpponentLogicModule(client);
		Optional<Player<?>> found = module.getPlayer(new FieldCoordinate(4, 4));
		assertTrue(found.isPresent());
		assertSame(homePlayer, found.get());
	}

	@Test
	public void testGetPlayerFallsBackToLastWhenNoHomeTeamPlayerFound() {
		given(client.getGame().getFieldModel().getPlayers(new FieldCoordinate(5, 5)))
			.willReturn(Arrays.asList(awayPlayer, player));
		given(client.getGame().getTeamHome().hasPlayer(awayPlayer)).willReturn(false);
		given(client.getGame().getTeamHome().hasPlayer(player)).willReturn(false);

		WaitForOpponentLogicModule module = new WaitForOpponentLogicModule(client);
		Optional<Player<?>> found = module.getPlayer(new FieldCoordinate(5, 5));
		assertTrue(found.isPresent());
		assertSame(player, found.get());
	}
}
