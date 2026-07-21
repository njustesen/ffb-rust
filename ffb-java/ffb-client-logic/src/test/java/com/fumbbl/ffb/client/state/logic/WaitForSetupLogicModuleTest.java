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
class WaitForSetupLogicModuleTest {

	@Mock
	private FantasyFootballClient client;

	@Mock
	private ActingPlayer actingPlayer;

	@Mock
	private Player<?> player;

	@Test
	public void testGetIdReturnsWaitForSetup() {
		WaitForSetupLogicModule module = new WaitForSetupLogicModule(client);
		assertEquals(ClientStateId.WAIT_FOR_SETUP, module.getId());
	}

	@Test
	public void testAvailableActionsIsEmpty() {
		assertTrue(new WaitForSetupLogicModule(client).availableActions().isEmpty());
	}

	@Test
	public void testPerformAvailableActionIsNoOp() {
		WaitForSetupLogicModule module = new WaitForSetupLogicModule(client);
		assertDoesNotThrow(() -> module.performAvailableAction(player, ClientAction.MOVE));
	}

	@Test
	public void testActionContextThrows() {
		WaitForSetupLogicModule module = new WaitForSetupLogicModule(client);
		UnsupportedOperationException exception = assertThrows(UnsupportedOperationException.class,
			() -> module.actionContext(actingPlayer));
		assertEquals("actionContext for acting player is not supported in waiting context", exception.getMessage());
	}
}
