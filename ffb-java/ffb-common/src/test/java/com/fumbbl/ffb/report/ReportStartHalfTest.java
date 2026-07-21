package com.fumbbl.ffb.report;

import com.eclipsesource.json.JsonObject;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

public class ReportStartHalfTest {

	private ReportStartHalf make() {
		return new ReportStartHalf(1);
	}

	@Test
	public void serializationRoundTrip() {
		ReportStartHalf original = make();
		JsonObject json = original.toJsonValue();
		ReportStartHalf restored = new ReportStartHalf().initFrom(ReportTestUtil.source(), json);
		assertEquals(original.getHalf(), restored.getHalf());
	}

	@Test
	public void toJsonValueHasReportId() {
		JsonObject json = make().toJsonValue();
		assertEquals("startHalf", json.get("reportId").asString());
	}
}
