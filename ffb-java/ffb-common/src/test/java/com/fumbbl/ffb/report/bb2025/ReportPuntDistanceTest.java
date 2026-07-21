package com.fumbbl.ffb.report.bb2025;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.report.ReportTestUtil;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

public class ReportPuntDistanceTest {
	private ReportPuntDistance make() {
		return new ReportPuntDistance(4, false);
	}

	@Test
	public void serializationRoundTrip() {
		ReportPuntDistance original = make();
		JsonObject json = original.toJsonValue();
		ReportPuntDistance restored = new ReportPuntDistance().initFrom(ReportTestUtil.source(), json);
		assertEquals(original.getRoll(), restored.getRoll());
		assertEquals(original.isOutOfBounds(), restored.isOutOfBounds());
	}

	@Test
	public void toJsonValueHasReportId() {
		JsonObject json = make().toJsonValue();
		assertEquals("puntDistanceRoll", json.get("reportId").asString());
	}
}
