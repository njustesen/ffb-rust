package com.fumbbl.ffb.util.pathfinding;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;

import java.util.HashSet;
import java.util.Set;

import com.fumbbl.ffb.FieldCoordinate;

import org.junit.jupiter.api.Test;

/**
 * Mirrors the surviving Rust tests in crates/ffb-model/src/util/pathfinding/path_find_node.rs
 * for {@link PathFindNode}.
 */
public class PathFindNodeTest {

	private static FieldCoordinate coord(int x, int y) {
		return new FieldCoordinate(x, y);
	}

	private static Set<FieldCoordinate> targetSet(int[][] coords) {
		Set<FieldCoordinate> set = new HashSet<>();
		for (int[] c : coords) {
			set.add(coord(c[0], c[1]));
		}
		return set;
	}

	@Test
	void getWeightIsDistancePlusEstimate() {
		Set<FieldCoordinate> targets = targetSet(new int[][] { { 5, 5 } });
		PathFindNode node = new PathFindNode(PathFindState.NORMAL, coord(3, 5), 2, false, targets, null);
		// estimate = distanceInSteps((3,5), (5,5)) = max(|2|, |0|) = 2
		assertEquals(4, node.getWeight());
	}

	@Test
	void estimateIs1000WhenNoTargets() {
		PathFindNode node = new PathFindNode(PathFindState.NORMAL, coord(0, 0), 0, false, null, null);
		assertEquals(1000, node.getWeight());
	}

	@Test
	void compareToOrdersByWeight() {
		Set<FieldCoordinate> targets = targetSet(new int[][] { { 10, 10 } });
		PathFindNode close = new PathFindNode(PathFindState.NORMAL, coord(9, 10), 1, false, targets, null);
		PathFindNode far = new PathFindNode(PathFindState.NORMAL, coord(5, 5), 1, false, targets, null);
		assertTrue(close.compareTo(far) < 0);
	}

	@Test
	void setSourceUpdatesDistanceAndPreservesState() {
		Set<FieldCoordinate> targets = targetSet(new int[][] { { 5, 5 } });
		PathFindNode parent = new PathFindNode(PathFindState.NORMAL, coord(0, 0), 0, false, null, null);
		PathFindNode child = new PathFindNode(PathFindState.NORMAL, coord(1, 0), 1, false, targets, null);
		child.setSource(parent, 2);
		assertEquals(2, child.getDistance());
		assertEquals(PathFindState.NORMAL, child.getState());
	}

	@Test
	void setSourceWithStateChangesState() {
		Set<FieldCoordinate> targets = targetSet(new int[][] { { 5, 5 } });
		PathFindNode parent = new PathFindNode(PathFindState.NORMAL, coord(0, 0), 0, false, null, null);
		PathFindNode child = new PathFindNode(PathFindState.NORMAL, coord(2, 0), 0, false, targets, null);
		child.setSource(parent, 2, PathFindState.HAS_JUMPED);
		assertEquals(PathFindState.HAS_JUMPED, child.getState());
		assertEquals(2, child.getDistance());
	}

	@Test
	void eqBasedOnWeights() {
		Set<FieldCoordinate> targets = targetSet(new int[][] { { 5, 5 } });
		PathFindNode a = new PathFindNode(PathFindState.NORMAL, coord(4, 5), 1, false, targets, null);
		PathFindNode b = new PathFindNode(PathFindState.NORMAL, coord(4, 5), 1, false, targets, null);
		assertEquals(a, b);
	}

}
