package com.fumbbl.ffb.report;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.Direction;
import com.fumbbl.ffb.FieldCoordinate;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

public class ReportPassDeviateTest {

	private ReportPassDeviate make() {
		return new ReportPassDeviate(new FieldCoordinate(10, 5), Direction.EAST, 6, 3, false);
	}

	@Test
	public void serializationRoundTrip() {
		ReportPassDeviate original = make();
		JsonObject json = original.toJsonValue();
		ReportPassDeviate restored = new ReportPassDeviate().initFrom(ReportTestUtil.source(), json);
		assertEquals(original.getBallCoordinateEnd(), restored.getBallCoordinateEnd());
		assertEquals(original.getScatterDirection(), restored.getScatterDirection());
		assertEquals(original.getRollScatterDirection(), restored.getRollScatterDirection());
		assertEquals(original.getRollScatterDistance(), restored.getRollScatterDistance());
		assertEquals(original.isTtm(), restored.isTtm());
	}

	@Test
	public void toJsonValueHasReportId() {
		JsonObject json = make().toJsonValue();
		assertEquals("passDeviate", json.get("reportId").asString());
	}
}
