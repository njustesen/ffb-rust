package com.fumbbl.ffb.report;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.Direction;
import com.fumbbl.ffb.FieldCoordinate;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

public class ReportKickoffScatterTest {

	private ReportKickoffScatter make() {
		return new ReportKickoffScatter(new FieldCoordinate(5, 7), Direction.NORTH, 3, 4);
	}

	@Test
	public void serializationRoundTrip() {
		ReportKickoffScatter original = make();
		JsonObject json = original.toJsonValue();
		ReportKickoffScatter restored = new ReportKickoffScatter().initFrom(ReportTestUtil.source(), json);
		assertEquals(original.getBallCoordinateEnd(), restored.getBallCoordinateEnd());
		assertEquals(original.getScatterDirection(), restored.getScatterDirection());
		assertEquals(original.getRollScatterDirection(), restored.getRollScatterDirection());
		assertEquals(original.getRollScatterDistance(), restored.getRollScatterDistance());
	}

	@Test
	public void toJsonValueHasReportId() {
		JsonObject json = make().toJsonValue();
		assertEquals("kickoffScatter", json.get("reportId").asString());
	}
}
