package com.fumbbl.ffb;

import com.eclipsesource.json.JsonValue;
import com.fumbbl.ffb.net.NetCommandTestUtil;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNull;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirrors the surviving Rust tests in
 * crates/ffb-model/src/types/range_ruler.rs for {@link RangeRuler}.
 */
public class RangeRulerTest {

	@Test
	public void rangeRulerMinimumRollDashForZero() {
		RangeRuler r = new RangeRuler("p1", new FieldCoordinate(5, 5), 0, false);
		assertEquals("--", r.getMinimumRoll());
	}

	@Test
	public void rangeRulerMinimumRoll3plusForRoll3() {
		RangeRuler r = new RangeRuler("p1", new FieldCoordinate(5, 5), 3, false);
		assertEquals("3+", r.getMinimumRoll());
	}

	@Test
	public void displayStrings() {
		assertEquals("--", new RangeRuler("p1", null, 0, false).getMinimumRoll());
		assertEquals("", new RangeRuler("p1", null, -1, false).getMinimumRoll());
		assertEquals("3+", new RangeRuler("p1", null, 3, false).getMinimumRoll());
		assertEquals("6", new RangeRuler("p1", null, 6, false).getMinimumRoll());
	}

	@Test
	public void transformMirrorsTarget() {
		RangeRuler r = new RangeRuler("p1", new FieldCoordinate(10, 7), 3, false);
		RangeRuler t = r.transform();
		assertEquals(25 - 10, t.getTargetCoordinate().getX());
		assertEquals("3+", t.getMinimumRoll());
	}

	@Test
	public void serdeRoundTrip() {
		RangeRuler r = new RangeRuler("p1", new FieldCoordinate(13, 7), 4, false);
		JsonValue json = r.toJsonValue();
		RangeRuler back = new RangeRuler().initFrom(NetCommandTestUtil.applicationSource(), json);
		assertEquals(r, back);
	}

	@Test
	public void displayStringsAllInRangeValues() {
		assertEquals("2+", new RangeRuler("p1", null, 2, false).getMinimumRoll());
		assertEquals("4+", new RangeRuler("p1", null, 4, false).getMinimumRoll());
		assertEquals("5+", new RangeRuler("p1", null, 5, false).getMinimumRoll());
		assertEquals("1+", new RangeRuler("p1", null, 1, false).getMinimumRoll());
	}

	@Test
	public void transformWithNoTargetCoordinate() {
		RangeRuler r = new RangeRuler("p2", null, 5, true);
		RangeRuler t = r.transform();
		assertNull(t.getTargetCoordinate());
		assertEquals("p2", t.getThrowerId());
		assertEquals("5+", t.getMinimumRoll());
		assertTrue(t.isThrowTeamMate());
	}
}
