package com.fumbbl.ffb;

import com.eclipsesource.json.JsonValue;
import com.fumbbl.ffb.net.NetCommandTestUtil;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirrors the surviving Rust tests in
 * crates/ffb-model/src/types/move_square.rs for {@link MoveSquare}.
 */
public class MoveSquareTest {

	@Test
	public void moveSquareKind() {
		FieldCoordinate c = new FieldCoordinate(5, 5);
		// Move: no dodge, no gfi
		assertFalse(new MoveSquare(c, 0, 0).isDodging());
		assertFalse(new MoveSquare(c, 0, 0).isGoingForIt());
		// Dodge only
		assertTrue(new MoveSquare(c, 3, 0).isDodging());
		assertFalse(new MoveSquare(c, 3, 0).isGoingForIt());
		// Rush (gfi) only
		assertFalse(new MoveSquare(c, 0, 2).isDodging());
		assertTrue(new MoveSquare(c, 0, 2).isGoingForIt());
		// Rush + dodge
		assertTrue(new MoveSquare(c, 3, 2).isDodging());
		assertTrue(new MoveSquare(c, 3, 2).isGoingForIt());
	}

	@Test
	public void transformMirrorsCoordinate() {
		MoveSquare sq = new MoveSquare(new FieldCoordinate(10, 7), 3, 0);
		MoveSquare t = sq.transform();
		assertEquals(25 - 10, t.getCoordinate().getX());
		assertEquals(3, t.getMinimumRollDodge());
	}

	@Test
	public void serdeRoundTrip() {
		MoveSquare sq = new MoveSquare(new FieldCoordinate(3, 5), 2, 0);
		JsonValue json = sq.toJsonValue();
		MoveSquare back = new MoveSquare().initFrom(NetCommandTestUtil.applicationSource(), json);
		assertEquals(sq.getCoordinate(), back.getCoordinate());
		assertEquals(sq.getMinimumRollDodge(), back.getMinimumRollDodge());
		assertEquals(sq.getMinimumRollGoForIt(), back.getMinimumRollGoForIt());
	}

	@Test
	public void moveSquareIsDodgingWhenDodgeRollPositive() {
		MoveSquare sq = new MoveSquare(new FieldCoordinate(5, 5), 3, 0);
		assertTrue(sq.isDodging());
	}

	@Test
	public void moveSquareNotDodgingWhenRollZero() {
		MoveSquare sq = new MoveSquare(new FieldCoordinate(5, 5), 0, 0);
		assertFalse(sq.isDodging());
	}

	@Test
	public void moveSquareIsGoingForItWhenGfiRollPositive() {
		MoveSquare sq = new MoveSquare(new FieldCoordinate(5, 5), 0, 2);
		assertTrue(sq.isGoingForIt());
		assertFalse(sq.isDodging());
	}

	@Test
	public void transformPreservesRolls() {
		MoveSquare sq = new MoveSquare(new FieldCoordinate(8, 3), 4, 2);
		MoveSquare t = sq.transform();
		assertEquals(4, t.getMinimumRollDodge());
		assertEquals(2, t.getMinimumRollGoForIt());
		assertEquals(3, t.getCoordinate().getY());
	}
}
