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

	// NOTE (test equalization): testGetIdReturnsWaitForSetup / testAvailableActionsIsEmpty /
	// testPerformAvailableActionIsNoOp pruned — trivial LogicModule-subclass boilerplate, no Rust twin.

	@Test
	public void testActionContextThrows() {
		WaitForSetupLogicModule module = new WaitForSetupLogicModule(client);
		UnsupportedOperationException exception = assertThrows(UnsupportedOperationException.class,
			() -> module.actionContext(actingPlayer));
		assertEquals("actionContext for acting player is not supported in waiting context", exception.getMessage());
	}
}
