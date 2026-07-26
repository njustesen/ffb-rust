package com.fumbbl.ffb.mechanics.bb2025;

import com.fumbbl.ffb.Direction;
import com.fumbbl.ffb.FieldCoordinate;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirror of ffb-rust crates/ffb-mechanics/src/bb2025/throw_in_mechanic.rs tests.
 */
public class ThrowInMechanicTest {

	private final ThrowInMechanic m = new ThrowInMechanic();

	// rust: distance_sums_two_dice
	@Test
	public void distanceSumsTwoDice() {
		assertEquals(7, m.distance(new int[]{2, 5}));
	}

	// rust: is_corner_throw_in_for_corner_coords
	@Test
	public void isCornerThrowInForCornerCoords() {
		assertTrue(m.isCornerThrowIn(new FieldCoordinate(0, 0)));
		assertTrue(m.isCornerThrowIn(new FieldCoordinate(25, 14)));
		assertFalse(m.isCornerThrowIn(new FieldCoordinate(0, 7)));
	}

	// rust: corner_northwest_roll_1_is_east
	@Test
	public void cornerNorthwestRoll1IsEast() {
		assertEquals(Direction.EAST, m.interpretThrowInDirectionRoll(new FieldCoordinate(0, 0), 1));
	}

	// rust: sideline_south_roll_3_is_east (asserts SOUTH, mirroring the Rust assertion)
	@Test
	public void sidelineSouthRoll3IsEast() {
		assertEquals(Direction.SOUTH, m.interpretThrowInDirectionRoll(new FieldCoordinate(12, 0), 3));
	}

	// rust: is_corner_throw_in_for_remaining_two_corners
	@Test
	public void isCornerThrowInForRemainingTwoCorners() {
		assertTrue(m.isCornerThrowIn(new FieldCoordinate(25, 0)));
		assertTrue(m.isCornerThrowIn(new FieldCoordinate(0, 14)));
	}
}
