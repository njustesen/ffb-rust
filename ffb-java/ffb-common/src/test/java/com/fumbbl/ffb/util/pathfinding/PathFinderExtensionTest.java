package com.fumbbl.ffb.util.pathfinding;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;

import java.util.Set;

import com.fumbbl.ffb.FieldCoordinate;
import com.fumbbl.ffb.FieldCoordinateBounds;

import org.junit.jupiter.api.Test;

/**
 * Mirrors the surviving Rust tests in crates/ffb-model/src/util/pathfinding/path_finder_extension.rs
 * for {@link PathFinderExtension}.
 *
 * <p>The Rust dimension_variance tests are omitted: dimensionVariance is private in the Java class.
 */
public class PathFinderExtensionTest {

	private static FieldCoordinate fc(int x, int y) {
		return new FieldCoordinate(x, y);
	}

	@Test
	void findPossiblePathSquaresDiagonalJump() {
		PathFinderExtension ext = new PathFinderExtension();
		// Jump from (5,5) to (7,7): delta=(2,2) -> each variance is [1], so only intermediate is (6,6)
		Set<FieldCoordinate> squares = ext.findPossiblePathSquares(fc(5, 5), fc(7, 7));
		assertTrue(squares.contains(fc(6, 6)));
		assertEquals(1, squares.size());
	}

	@Test
	void findPossiblePathSquaresOrthogonalJump() {
		PathFinderExtension ext = new PathFinderExtension();
		// Jump from (5,5) to (7,5): delta=(2,0) -> x_variance=[1], y_variance=[0] -> intermediate (6,5)
		Set<FieldCoordinate> squares = ext.findPossiblePathSquares(fc(5, 5), fc(7, 5));
		assertTrue(squares.contains(fc(6, 5)));
		assertEquals(1, squares.size());
	}

	@Test
	void findPossiblePathSquaresFiltersOutOfBounds() {
		PathFinderExtension ext = new PathFinderExtension();
		// Jump from (0,0) to (2,0): intermediate is (1,0) - in bounds
		Set<FieldCoordinate> squares = ext.findPossiblePathSquares(fc(0, 0), fc(2, 0));
		for (FieldCoordinate sq : squares) {
			assertTrue(FieldCoordinateBounds.FIELD.isInBounds(sq));
		}
	}

	@Test
	void findPossiblePathSquaresAdjacentDelta1() {
		PathFinderExtension ext = new PathFinderExtension();
		// Jump from (5,5) to (6,7): delta=(1,2) -> x_var=[1,0], y_var=[1] -> coords (6,6) and (5,6)
		Set<FieldCoordinate> squares = ext.findPossiblePathSquares(fc(5, 5), fc(6, 7));
		assertTrue(squares.contains(fc(6, 6)));
		assertTrue(squares.contains(fc(5, 6)));
	}

}
