package com.fumbbl.ffb.report;

import com.eclipsesource.json.JsonObject;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

public class ReportHandOverTest {

	private ReportHandOver make() {
		return new ReportHandOver("catcher1");
	}

	@Test
	public void serializationRoundTrip() {
		ReportHandOver original = make();
		JsonObject json = original.toJsonValue();
		ReportHandOver restored = new ReportHandOver().initFrom(ReportTestUtil.source(), json);
		assertEquals(original.getCatcherId(), restored.getCatcherId());
	}

	@Test
	public void toJsonValueHasReportId() {
		JsonObject json = make().toJsonValue();
		assertEquals("handOver", json.get("reportId").asString());
	}
}
