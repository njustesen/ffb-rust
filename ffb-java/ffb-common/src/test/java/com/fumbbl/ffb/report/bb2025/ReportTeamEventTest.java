package com.fumbbl.ffb.report.bb2025;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.report.ReportTestUtil;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

public class ReportTeamEventTest {
	private ReportTeamEvent make() {
		return new ReportTeamEvent("team1", "Player banned!");
	}

	@Test
	public void serializationRoundTrip() {
		ReportTeamEvent original = make();
		JsonObject json = original.toJsonValue();
		ReportTeamEvent restored = new ReportTeamEvent().initFrom(ReportTestUtil.source(), json);
		assertEquals(original.getTeamId(), restored.getTeamId());
		assertEquals(original.getEventMessage(), restored.getEventMessage());
	}

	@Test
	public void toJsonValueHasReportId() {
		JsonObject json = make().toJsonValue();
		assertEquals("teamEvent", json.get("reportId").asString());
	}
}
