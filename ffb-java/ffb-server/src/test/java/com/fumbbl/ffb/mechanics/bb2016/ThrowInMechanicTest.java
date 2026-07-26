package com.fumbbl.ffb.mechanics.bb2016;

import com.fumbbl.ffb.Direction;
import com.fumbbl.ffb.FieldCoordinate;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;

/**
 * Mirror of ffb-rust crates/ffb-mechanics/src/bb2016/throw_in_mechanic.rs tests.
 */
public class ThrowInMechanicTest {

	private final ThrowInMechanic m = new ThrowInMechanic();

	// rust: distance_sums_two_dice
	@Test
	public void distanceSumsTwoDice() {
		assertEquals(7, m.distance(new int[]{3, 4}));
	}

	// rust: is_corner_throw_in_always_false
	@Test
	public void isCornerThrowInAlwaysFalse() {
		assertFalse(m.isCornerThrowIn(new FieldCoordinate(0, 0)));
	}

	// rust: direction_from_west_edge_roll_3_is_east
	@Test
	public void directionFromWestEdgeRoll3IsEast() {
		assertEquals(Direction.EAST, m.interpretThrowInDirectionRoll(new FieldCoordinate(0, 7), 3));
	}

	// rust: direction_from_east_edge_roll_1_is_southwest
	@Test
	public void directionFromEastEdgeRoll1IsSouthwest() {
		assertEquals(Direction.SOUTHWEST, m.interpretThrowInDirectionRoll(new FieldCoordinate(25, 7), 1));
	}

	// rust: direction_from_north_edge_roll_1_is_northwest
	@Test
	public void directionFromNorthEdgeRoll1IsNorthwest() {
		assertEquals(Direction.NORTHWEST, m.interpretThrowInDirectionRoll(new FieldCoordinate(12, 14), 1));
	}
}
