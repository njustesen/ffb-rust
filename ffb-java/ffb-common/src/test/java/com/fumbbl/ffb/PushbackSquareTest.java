package com.fumbbl.ffb;

import com.eclipsesource.json.JsonValue;
import com.fumbbl.ffb.net.NetCommandTestUtil;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;

/**
 * Mirrors the surviving Rust tests in
 * crates/ffb-model/src/types/pushback_square.rs for {@link PushbackSquare}.
 */
public class PushbackSquareTest {

	@Test
	public void pushbackSquareHomeChoiceFlipsOnTransform() {
		PushbackSquare sq = new PushbackSquare(new FieldCoordinate(10, 7), Direction.EAST, true);
		PushbackSquare t = sq.transform();
		assertFalse(t.isHomeChoice());
		assertEquals(FieldCoordinate.FIELD_WIDTH - 1 - 10, t.getCoordinate().getX());
	}

	@Test
	public void pushbackSquareTransformMirrorsDirection() {
		PushbackSquare sq = new PushbackSquare(new FieldCoordinate(10, 7), Direction.EAST, true);
		PushbackSquare t = sq.transform();
		assertEquals(Direction.WEST, t.getDirection());
	}

	@Test
	public void serdeRoundTrip() {
		PushbackSquare sq = new PushbackSquare(new FieldCoordinate(5, 5), Direction.NORTH, false);
		JsonValue json = sq.toJsonValue();
		PushbackSquare back = new PushbackSquare().initFrom(NetCommandTestUtil.applicationSource(), json);
		assertEquals(sq.getCoordinate(), back.getCoordinate());
		assertEquals(sq.getDirection(), back.getDirection());
		assertEquals(sq.isHomeChoice(), back.isHomeChoice());
	}

	@Test
	public void transformDoubleInvertsHomeChoice() {
		PushbackSquare sq = new PushbackSquare(new FieldCoordinate(10, 7), Direction.EAST, true);
		PushbackSquare t = sq.transform().transform();
		assertEquals(sq.isHomeChoice(), t.isHomeChoice());
	}
}
