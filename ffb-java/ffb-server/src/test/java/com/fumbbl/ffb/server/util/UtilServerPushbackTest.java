package com.fumbbl.ffb.server.util;

import com.fumbbl.ffb.Direction;
import com.fumbbl.ffb.FieldCoordinate;
import com.fumbbl.ffb.PushbackSquare;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNull;

/**
 * Mirror of ffb-rust crates/ffb-engine/src/util/util_server_pushback.rs findStartingSquare tests.
 * findStartingSquare maps the attacker->defender delta to the push direction (the square is
 * anchored on the defender's coordinate); equal coordinates return null.
 */
public class UtilServerPushbackTest {

	private PushbackSquare start(int sx, int sy, int ex, int ey) {
		return UtilServerPushback.findStartingSquare(
			new FieldCoordinate(sx, sy), new FieldCoordinate(ex, ey), true);
	}

	// rust: find_starting_square_attacker_north_of_defender (start (5,5) -> end (5,6): south)
	@Test
	public void attackerNorthOfDefenderPushesSouth() {
		PushbackSquare sq = start(5, 5, 5, 6);
		assertEquals(Direction.SOUTH, sq.getDirection());
		assertEquals(new FieldCoordinate(5, 6), sq.getCoordinate());
		assertEquals(true, sq.isHomeChoice());
	}

	// rust: find_starting_square_attacker_south_of_defender (start (5,7) -> end (5,6): north)
	@Test
	public void attackerSouthOfDefenderPushesNorth() {
		assertEquals(Direction.NORTH, start(5, 7, 5, 6).getDirection());
	}

	// rust: find_starting_square_attacker_east_of_defender (start (6,5) -> end (5,5): west)
	@Test
	public void attackerEastOfDefenderPushesWest() {
		assertEquals(Direction.WEST, start(6, 5, 5, 5).getDirection());
	}

	// rust: find_starting_square_attacker_west_of_defender (start (4,5) -> end (5,5): east)
	@Test
	public void attackerWestOfDefenderPushesEast() {
		assertEquals(Direction.EAST, start(4, 5, 5, 5).getDirection());
	}

	// rust: find_starting_square_diagonal_northeast (start (5,7) -> end (6,6): northeast)
	@Test
	public void diagonalNortheast() {
		assertEquals(Direction.NORTHEAST, start(5, 7, 6, 6).getDirection());
	}

	// rust: find_starting_square_same_square_returns_none
	@Test
	public void sameSquareReturnsNull() {
		assertNull(start(5, 7, 5, 7));
	}
}
