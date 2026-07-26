package com.fumbbl.ffb.mechanics.bb2020;

import com.fumbbl.ffb.Direction;
import com.fumbbl.ffb.FieldCoordinate;
import com.fumbbl.ffb.mechanics.Mechanic;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;

/**
 * Mirror of ffb-rust crates/ffb-mechanics/src/bb2020/throw_in_mechanic.rs tests.
 */
public class ThrowInMechanicTest {

	private final ThrowInMechanic m = new ThrowInMechanic();

	// rust: distance_sums_two_dice_plus_one
	@Test
	public void distanceSumsTwoDicePlusOne() {
		assertEquals(8, m.distance(new int[]{3, 4}));
	}

	// rust: direction_from_south_edge_roll_1_is_southeast
	@Test
	public void directionFromSouthEdgeRoll1IsSoutheast() {
		assertEquals(Direction.SOUTHEAST, m.interpretThrowInDirectionRoll(new FieldCoordinate(12, 0), 1));
	}

	// rust: direction_from_north_edge_roll_4_is_north
	@Test
	public void directionFromNorthEdgeRoll4IsNorth() {
		assertEquals(Direction.NORTH, m.interpretThrowInDirectionRoll(new FieldCoordinate(12, 14), 4));
	}

	// rust: distance_minimum_dice
	@Test
	public void distanceMinimumDice() {
		assertEquals(3, m.distance(new int[]{1, 1}));
	}

	// rust: distance_maximum_dice
	@Test
	public void distanceMaximumDice() {
		assertEquals(13, m.distance(new int[]{6, 6}));
	}

	// rust: is_corner_throw_in_always_false
	@Test
	public void isCornerThrowInAlwaysFalse() {
		assertFalse(m.isCornerThrowIn(new FieldCoordinate(0, 0)));
		assertFalse(m.isCornerThrowIn(new FieldCoordinate(24, 14)));
	}

	// rust: direction_from_east_edge
	@Test
	public void directionFromEastEdge() {
		assertEquals(Direction.WEST, m.interpretThrowInDirectionRoll(new FieldCoordinate(25, 7), 4));
	}

	// rust: mechanic_type_is_throw_in
	@Test
	public void mechanicTypeIsThrowIn() {
		assertEquals(Mechanic.Type.THROW_IN, m.getType());
	}
}
