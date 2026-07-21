package com.fumbbl.ffb.report.mixed;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.report.ReportTestUtil;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

public class ReportPlayerEventTest {

	private ReportPlayerEvent make() {
		return new ReportPlayerEvent("p1", "some event");
	}

	@Test
	public void serializationRoundTrip() {
		ReportPlayerEvent original = make();
		JsonObject json = original.toJsonValue().asObject();
		ReportPlayerEvent restored = new ReportPlayerEvent().initFrom(ReportTestUtil.source(), json);
		assertEquals(original.getPlayerId(), restored.getPlayerId());
		assertEquals(original.getEventMessage(), restored.getEventMessage());
	}

	@Test
	public void toJsonValueHasReportId() {
		JsonObject json = make().toJsonValue().asObject();
		assertEquals("playerEvent", json.get("reportId").asString());
	}
}
