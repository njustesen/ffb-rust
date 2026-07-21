package com.fumbbl.ffb.util.pathfinding;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertNotNull;
import static org.junit.jupiter.api.Assertions.assertNull;
import static org.junit.jupiter.api.Assertions.assertSame;
import static org.junit.jupiter.api.Assertions.assertTrue;

import com.fumbbl.ffb.FieldCoordinate;

import org.junit.jupiter.api.Test;

/**
 * Mirrors the surviving Rust tests in crates/ffb-model/src/util/pathfinding/path_find_data.rs
 * for {@link PathFindData}.
 */
public class PathFindDataTest {

	private static FieldCoordinate fc(int x, int y) {
		return new FieldCoordinate(x, y);
	}

	private static PathFindNode makeNode(PathFindState state, FieldCoordinate coord, int dist) {
		return new PathFindNode(state, coord, dist, false, null, null);
	}

	@Test
	void setNodeNormalMarksProcessed() {
		PathFindData data = new PathFindData();
		PathFindNode node = makeNode(PathFindState.NORMAL, fc(3, 4), 1);
		data.setNode(fc(3, 4), node);
		assertTrue(data.isProcessed(PathFindState.NORMAL, 3, 4));
		assertFalse(data.isProcessed(PathFindState.HAS_JUMPED, 3, 4));
	}

	@Test
	void setNodeJumpedMarksProcessedInJumpedSlot() {
		PathFindData data = new PathFindData();
		PathFindNode node = makeNode(PathFindState.HAS_JUMPED, fc(1, 1), 2);
		data.setNode(fc(1, 1), node);
		assertFalse(data.isProcessed(PathFindState.NORMAL, 1, 1));
		assertTrue(data.isProcessed(PathFindState.HAS_JUMPED, 1, 1));
	}

	@Test
	void blockNodeMarksBothStates() {
		PathFindData data = new PathFindData();
		data.blockNode(fc(5, 5));
		assertTrue(data.isProcessed(PathFindState.NORMAL, 5, 5));
		assertTrue(data.isProcessed(PathFindState.HAS_JUMPED, 5, 5));
	}

	@Test
	void getNeighbourReturnsSetNode() {
		PathFindData data = new PathFindData();
		PathFindNode node = makeNode(PathFindState.NORMAL, fc(2, 3), 1);
		data.setNode(fc(2, 3), node);
		PathFindNode found = data.getNeighbour(PathFindState.NORMAL, fc(2, 3));
		assertNotNull(found);
		assertEquals(fc(2, 3), found.getCoord());
	}

	@Test
	void getNeighbourReturnsNoneForUnsetCoord() {
		PathFindData data = new PathFindData();
		assertNull(data.getNeighbour(PathFindState.NORMAL, fc(10, 7)));
	}

	@Test
	void blockNodeReturnsSameRefInBothSlots() {
		PathFindData data = new PathFindData();
		PathFindNode blocked = data.blockNode(fc(4, 4));
		PathFindNode fromNormal = data.getNeighbour(PathFindState.NORMAL, fc(4, 4));
		PathFindNode fromJumped = data.getNeighbour(PathFindState.HAS_JUMPED, fc(4, 4));
		assertSame(blocked, fromNormal);
		assertSame(blocked, fromJumped);
	}

}
