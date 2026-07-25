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

	// NOTE (test equalization): testGetIdReturnsQuickSnap / testAvailableActionsIsEmpty /
	// testActionContextThrowsViaDelegation pruned — trivial LogicModule-subclass boilerplate with no
	// Rust twin (Rust QuickSnapLogicModule tests only the behavioral useTurnMode/squareIsOnPitch/
	// squaresAreSameOrAdjacent cases). Rust-as-reference.

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

}
