package com.fumbbl.ffb.report.bb2016;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.Direction;
import com.fumbbl.ffb.FieldCoordinate;
import com.fumbbl.ffb.report.ReportTestUtil;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertArrayEquals;
import static org.junit.jupiter.api.Assertions.assertEquals;

public class ReportSwoopPlayerTest {

	private ReportSwoopPlayer make() {
		return new ReportSwoopPlayer(new FieldCoordinate(5, 7), new FieldCoordinate(8, 7),
			new Direction[]{Direction.NORTH, Direction.EAST}, new int[]{3, 5});
	}

	@Test
	public void serializationRoundTrip() {
		ReportSwoopPlayer original = make();
		JsonObject json = original.toJsonValue();
		ReportSwoopPlayer restored = new ReportSwoopPlayer().initFrom(ReportTestUtil.source(), json);
		assertEquals(original.getStartCoordinate().getX(), restored.getStartCoordinate().getX());
		assertArrayEquals(original.getRolls(), restored.getRolls());
		assertEquals(original.getDirections().length, restored.getDirections().length);
	}

	@Test
	public void toJsonValueHasReportId() {
		JsonObject json = make().toJsonValue();
		assertEquals("swoopPlayer", json.get("reportId").asString());
	}
}
