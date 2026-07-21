package com.fumbbl.ffb.client.state.logic;

import com.fumbbl.ffb.ClientStateId;
import com.fumbbl.ffb.FieldCoordinate;
import com.fumbbl.ffb.client.FantasyFootballClient;
import com.fumbbl.ffb.model.ActingPlayer;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;

@ExtendWith(MockitoExtension.class)
class QuickSnapLogicModuleTest {

	@Mock
	private FantasyFootballClient client;

	@Mock
	private ActingPlayer actingPlayer;

	@Test
	public void testGetIdReturnsQuickSnap() {
		QuickSnapLogicModule module = new QuickSnapLogicModule(client);
		assertEquals(ClientStateId.QUICK_SNAP, module.getId());
	}

	@Test
	public void testUseTurnModeIsTrue() {
		// useTurnMode() is protected; same-package test classes can call it directly.
		assertTrue(new QuickSnapLogicModule(client).useTurnMode());
	}

	@Test
	public void testSquareIsOnPitchFalseForBoxCoordinate() {
		QuickSnapLogicModule module = new QuickSnapLogicModule(client);
		assertFalse(module.squareIsOnPitch(null));
		assertTrue(module.squareIsOnPitch(new FieldCoordinate(5, 5)));
		assertFalse(module.squareIsOnPitch(new FieldCoordinate(FieldCoordinate.RSV_HOME_X, 3)));
	}

	@Test
	public void testSquaresAreSameOrAdjacentChecksEqualityAndAdjacency() {
		QuickSnapLogicModule module = new QuickSnapLogicModule(client);
		FieldCoordinate a = new FieldCoordinate(5, 5);
		FieldCoordinate b = new FieldCoordinate(6, 5);
		FieldCoordinate far = new FieldCoordinate(10, 10);
		assertTrue(module.squaresAreSameOrAdjacent(a, a));
		assertTrue(module.squaresAreSameOrAdjacent(a, b));
		assertFalse(module.squaresAreSameOrAdjacent(a, far));
		assertFalse(module.squaresAreSameOrAdjacent(null, a));
	}

	@Test
	public void testAvailableActionsIsEmpty() {
		assertTrue(new QuickSnapLogicModule(client).availableActions().isEmpty());
	}

	@Test
	public void testActionContextThrowsViaDelegation() {
		QuickSnapLogicModule module = new QuickSnapLogicModule(client);
		UnsupportedOperationException exception = assertThrows(UnsupportedOperationException.class,
			() -> module.actionContext(actingPlayer));
		assertEquals("actionContext for acting player is not supported in setup context", exception.getMessage());
	}
}
