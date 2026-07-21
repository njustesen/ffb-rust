package com.fumbbl.ffb;

import com.eclipsesource.json.JsonArray;
import com.eclipsesource.json.JsonObject;
import com.eclipsesource.json.JsonValue;
import com.fumbbl.ffb.json.UtilJson;
import com.fumbbl.ffb.net.NetCommandTestUtil;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertNull;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirrors the surviving Rust tests in
 * crates/ffb-model/src/types/field_coordinate.rs for {@link FieldCoordinate}.
 */
public class FieldCoordinateTest {

	@Test
	public void toJsonValueIsTwoElementArray() {
		FieldCoordinate fc = new FieldCoordinate(4, 9);
		JsonValue json = UtilJson.toJsonValue(fc);
		assertTrue(json.isArray());
		JsonArray array = json.asArray();
		assertEquals(2, array.size());
		assertEquals(4, array.get(0).asInt());
		assertEquals(9, array.get(1).asInt());
	}

	@Test
	public void fromJsonRoundTrip() {
		FieldCoordinate fc = new FieldCoordinate(11, 2);
		JsonValue json = UtilJson.toJsonValue(fc);
		assertEquals(fc, UtilJson.toFieldCoordinate(json));
	}

	@Test
	public void fromJsonRejectsWrongShape() {
		JsonArray tooShort = new JsonArray();
		tooShort.add(1);
		assertThrows(IllegalArgumentException.class, () -> UtilJson.toFieldCoordinate(tooShort));
		JsonObject asObject = new JsonObject();
		asObject.add("x", 1);
		asObject.add("y", 2);
		assertThrows(IllegalArgumentException.class, () -> UtilJson.toFieldCoordinate(asObject));
	}

	@Test
	public void distanceChebyshev() {
		FieldCoordinate a = new FieldCoordinate(0, 0);
		FieldCoordinate b = new FieldCoordinate(3, 1);
		assertEquals(3, a.distanceInSteps(b));
	}

	@Test
	public void fieldCoordinateAdjacentToNeighbor() {
		assertTrue(new FieldCoordinate(5, 5).isAdjacent(new FieldCoordinate(6, 5)));
		assertTrue(new FieldCoordinate(5, 5).isAdjacent(new FieldCoordinate(5, 6)));
	}

	@Test
	public void fieldCoordinateAdjacentToDiagonal() {
		assertTrue(new FieldCoordinate(5, 5).isAdjacent(new FieldCoordinate(6, 6)));
	}

	@Test
	public void fieldCoordinateNotAdjacentToFarSquare() {
		assertFalse(new FieldCoordinate(1, 1).isAdjacent(new FieldCoordinate(5, 5)));
	}

	@Test
	public void transformMirrorsField() {
		FieldCoordinate c = new FieldCoordinate(10, 7);
		FieldCoordinate t = c.transform();
		assertEquals(FieldCoordinate.FIELD_WIDTH - 1 - 10, t.getX());
		assertEquals(7, t.getY());
		assertEquals(c, t.transform());
	}

	@Test
	public void transformDugout() {
		FieldCoordinate homeRsv = new FieldCoordinate(FieldCoordinate.RSV_HOME_X, 3);
		FieldCoordinate awayRsv = homeRsv.transform();
		assertEquals(FieldCoordinate.RSV_AWAY_X, awayRsv.getX());
		assertEquals(FieldCoordinate.RSV_HOME_X, awayRsv.transform().getX());
	}

	@Test
	public void directionTo() {
		FieldCoordinate origin = new FieldCoordinate(5, 5);
		assertEquals(Direction.NORTHEAST, origin.getDirection(new FieldCoordinate(6, 4)));
		assertNull(origin.getDirection(origin));
	}

	@Test
	public void serdeRoundTrip() {
		FieldCoordinate c = new FieldCoordinate(13, 7);
		JsonValue json = c.toJsonValue();
		FieldCoordinate back = new FieldCoordinate().initFrom(NetCommandTestUtil.applicationSource(), json);
		assertEquals(c, back);
	}

	@Test
	public void fieldCoordinateAddReturnsCorrectResult() {
		FieldCoordinate c = new FieldCoordinate(5, 5).add(2, -1);
		assertEquals(7, c.getX());
		assertEquals(4, c.getY());
	}

	@Test
	public void fieldCoordinateHomeBoxIsBoxCoordinate() {
		assertTrue(new FieldCoordinate(FieldCoordinate.RSV_HOME_X, 1).isBoxCoordinate());
		assertTrue(new FieldCoordinate(FieldCoordinate.KO_HOME_X, 1).isBoxCoordinate());
	}

	@Test
	public void fieldCoordinateAwayBoxIsBoxCoordinate() {
		assertTrue(new FieldCoordinate(FieldCoordinate.RSV_AWAY_X, 1).isBoxCoordinate());
	}

	@Test
	public void fieldCoordinatePitchCoordinateIsNotBox() {
		assertFalse(new FieldCoordinate(5, 5).isBoxCoordinate());
	}

	@Test
	public void stepMovesByDirectionAndDistance() {
		FieldCoordinate c = new FieldCoordinate(5, 5);
		FieldCoordinate east2 = c.move(Direction.EAST, 2);
		assertEquals(7, east2.getX());
		assertEquals(5, east2.getY());
		FieldCoordinate south1 = c.move(Direction.SOUTH, 1);
		assertEquals(5, south1.getX());
		assertEquals(6, south1.getY());
	}
}
