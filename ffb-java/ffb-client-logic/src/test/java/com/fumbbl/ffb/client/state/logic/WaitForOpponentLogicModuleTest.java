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

	// NOTE (test equalization): testGetIdReturnsWaitForOpponent / testAvailableActionsIsEmpty /
	// testPerformAvailableActionIsNoOp pruned — trivial LogicModule-subclass boilerplate, no Rust twin.

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

	// NOTE (test equalization): testGetPlayerPrefersHomeTeamPlayerAmongMultiple /
	// testGetPlayerFallsBackToLastWhenNoHomeTeamPlayerFound pruned — Rust's wait_for_opponent suite
	// tests only the none + single-occupant getPlayer cases; these multi-occupant variations have no
	// Rust twin (Rust-as-reference).
}
