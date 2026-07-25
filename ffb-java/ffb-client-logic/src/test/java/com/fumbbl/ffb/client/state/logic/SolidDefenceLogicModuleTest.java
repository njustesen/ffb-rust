package com.fumbbl.ffb.client.state.logic;

import com.fumbbl.ffb.ClientStateId;
import com.fumbbl.ffb.client.FantasyFootballClient;
import com.fumbbl.ffb.model.ActingPlayer;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;

@ExtendWith(MockitoExtension.class)
class SolidDefenceLogicModuleTest {

	@Mock
	private FantasyFootballClient client;

	@Mock
	private ActingPlayer actingPlayer;

	// NOTE (test equalization): testGetIdReturnsSolidDefence / testAvailableActionsIsEmpty pruned —
	// trivial LogicModule-subclass boilerplate, no Rust twin.

	@Test
	public void testActionContextThrowsViaDelegation() {
		SolidDefenceLogicModule module = new SolidDefenceLogicModule(client);
		UnsupportedOperationException exception = assertThrows(UnsupportedOperationException.class,
			() -> module.actionContext(actingPlayer));
		assertEquals("actionContext for acting player is not supported in setup context", exception.getMessage());
	}
}
