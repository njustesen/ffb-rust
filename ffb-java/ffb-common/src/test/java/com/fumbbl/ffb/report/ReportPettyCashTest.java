package com.fumbbl.ffb.report;

import com.eclipsesource.json.JsonObject;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

public class ReportPettyCashTest {

	private ReportPettyCash make() {
		return new ReportPettyCash("team1", 50);
	}

	@Test
	public void serializationRoundTrip() {
		ReportPettyCash original = make();
		JsonObject json = original.toJsonValue();
		ReportPettyCash restored = new ReportPettyCash().initFrom(ReportTestUtil.source(), json);
		assertEquals(original.getTeamId(), restored.getTeamId());
		assertEquals(original.getGold(), restored.getGold());
	}

	@Test
	public void toJsonValueHasReportId() {
		JsonObject json = make().toJsonValue();
		assertEquals("pettyCash", json.get("reportId").asString());
	}
}
