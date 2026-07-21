package com.fumbbl.ffb.report;

import com.eclipsesource.json.JsonObject;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

public class ReportBombOutOfBoundsTest {

	private ReportBombOutOfBounds make() {
		return new ReportBombOutOfBounds();
	}

	@Test
	public void serializationRoundTrip() {
		ReportBombOutOfBounds original = make();
		JsonObject json = original.toJsonValue();
		ReportBombOutOfBounds restored = new ReportBombOutOfBounds().initFrom(ReportTestUtil.source(), json);
		assertEquals(original.getId(), restored.getId());
	}

	@Test
	public void toJsonValueHasReportId() {
		JsonObject json = make().toJsonValue();
		assertEquals("bombOutOfBounds", json.get("reportId").asString());
	}
}
