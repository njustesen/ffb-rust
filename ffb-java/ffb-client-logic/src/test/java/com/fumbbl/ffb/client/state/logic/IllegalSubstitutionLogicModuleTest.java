package com.fumbbl.ffb.client.state.logic;

import com.fumbbl.ffb.ClientStateId;
import com.fumbbl.ffb.client.FantasyFootballClient;
import com.fumbbl.ffb.model.ActingPlayer;
import com.fumbbl.ffb.model.Player;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;

import static org.junit.jupiter.api.Assertions.assertDoesNotThrow;
import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;

@ExtendWith(MockitoExtension.class)
class IllegalSubstitutionLogicModuleTest {

	@Mock
	private FantasyFootballClient client;

	@Mock
	private ActingPlayer actingPlayer;

	@Mock
	private Player<?> player;

	@Test
	public void testGetIdReturnsIllegalSubstitution() {
		IllegalSubstitutionLogicModule module = new IllegalSubstitutionLogicModule(client);
		assertEquals(ClientStateId.ILLEGAL_SUBSTITUTION, module.getId());
	}

	@Test
	public void testAvailableActionsDelegatesToSetupAndIsEmpty() {
		assertTrue(new IllegalSubstitutionLogicModule(client).availableActions().isEmpty());
	}

	@Test
	public void testPerformAvailableActionIsNoOp() {
		IllegalSubstitutionLogicModule module = new IllegalSubstitutionLogicModule(client);
		assertDoesNotThrow(() -> module.performAvailableAction(player, ClientAction.MOVE));
	}

	@Test
	public void testActionContextThrowsViaDelegation() {
		IllegalSubstitutionLogicModule module = new IllegalSubstitutionLogicModule(client);
		UnsupportedOperationException exception = assertThrows(UnsupportedOperationException.class,
			() -> module.actionContext(actingPlayer));
		assertEquals("actionContext for acting player is not supported in setup context", exception.getMessage());
	}

	// SKIPPED: setUp() populates the private fFieldPlayers set from
	// client.getGame().getTeamHome().getPlayers() and FieldModel coordinates for each player —
	// building this graph requires a live Game/Team/Player mock chain rather than plain values.
	// SKIPPED: isSubstitute(Player) / squareContainsSubstitute(FieldCoordinate) both read the
	// private fFieldPlayers field that only setUp() can populate, so exercising them requires the
	// same live-game-state construction as setUp() above.
}
