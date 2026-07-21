package com.fumbbl.ffb.report.mixed;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.report.ReportTestUtil;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

public class ReportEventTest {

	private ReportEvent make() {
		return new ReportEvent("something happened");
	}

	@Test
	public void serializationRoundTrip() {
		ReportEvent original = make();
		JsonObject json = (JsonObject) original.toJsonValue();
		ReportEvent restored = new ReportEvent().initFrom(ReportTestUtil.source(), json);
		assertEquals(original.getEventMessage(), restored.getEventMessage());
	}

	@Test
	public void toJsonValueHasReportId() {
		JsonObject json = (JsonObject) make().toJsonValue();
		assertEquals("event", json.get("reportId").asString());
	}
}
