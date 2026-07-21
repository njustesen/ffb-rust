package com.fumbbl.ffb.report.bb2020;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.Direction;
import com.fumbbl.ffb.FieldCoordinate;
import com.fumbbl.ffb.report.ReportTestUtil;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

public class ReportSwoopPlayerTest {
	private ReportSwoopPlayer make() {
		return new ReportSwoopPlayer(new FieldCoordinate(5, 7), new FieldCoordinate(8, 7), Direction.EAST, 3);
	}

	@Test
	public void serializationRoundTrip() {
		ReportSwoopPlayer original = make();
		JsonObject json = original.toJsonValue();
		ReportSwoopPlayer restored = new ReportSwoopPlayer().initFrom(ReportTestUtil.source(), json);
		assertEquals(original.getStartCoordinate().getX(), restored.getStartCoordinate().getX());
		assertEquals(original.getStartCoordinate().getY(), restored.getStartCoordinate().getY());
		assertEquals(original.getEndCoordinate().getX(), restored.getEndCoordinate().getX());
		assertEquals(original.getEndCoordinate().getY(), restored.getEndCoordinate().getY());
		assertEquals(original.getDirection(), restored.getDirection());
		assertEquals(original.getDistance(), restored.getDistance());
	}

	@Test
	public void toJsonValueHasReportId() {
		JsonObject json = make().toJsonValue();
		assertEquals("swoopPlayer", json.get("reportId").asString());
	}
}
