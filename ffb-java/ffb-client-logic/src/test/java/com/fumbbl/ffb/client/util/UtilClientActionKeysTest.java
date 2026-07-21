package com.fumbbl.ffb.client.util;

import com.fumbbl.ffb.Direction;
import com.fumbbl.ffb.FieldCoordinate;
import com.fumbbl.ffb.client.ActionKey;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNull;

class UtilClientActionKeysTest {

	@Test
	void findMoveDirectionMapsAllEight() {
		assertEquals(Direction.NORTH, UtilClientActionKeys.findMoveDirection(ActionKey.PLAYER_MOVE_NORTH));
		assertEquals(Direction.SOUTHWEST, UtilClientActionKeys.findMoveDirection(ActionKey.PLAYER_MOVE_SOUTHWEST));
	}

	@Test
	void findMoveDirectionNonMoveKeyIsNone() {
		assertNull(UtilClientActionKeys.findMoveDirection(ActionKey.PLAYER_ACTION_BLOCK));
	}

	@Test
	void findMoveCoordinateNorthDecrementsY() {
		FieldCoordinate start = new FieldCoordinate(5, 5);
		FieldCoordinate result = UtilClientActionKeys.findMoveCoordinate(start, ActionKey.PLAYER_MOVE_NORTH);
		assertEquals(new FieldCoordinate(5, 4), result);
	}

	@Test
	void findMoveCoordinateSoutheastIncrementsBoth() {
		FieldCoordinate start = new FieldCoordinate(5, 5);
		FieldCoordinate result = UtilClientActionKeys.findMoveCoordinate(start, ActionKey.PLAYER_MOVE_SOUTHEAST);
		assertEquals(new FieldCoordinate(6, 6), result);
	}

	@Test
	void findMoveCoordinateNonMoveKeyIsNone() {
		FieldCoordinate start = new FieldCoordinate(5, 5);
		assertNull(UtilClientActionKeys.findMoveCoordinate(start, ActionKey.PLAYER_ACTION_PASS));
	}
}
