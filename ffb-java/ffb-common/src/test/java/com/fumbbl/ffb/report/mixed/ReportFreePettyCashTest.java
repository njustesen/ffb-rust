package com.fumbbl.ffb.report.mixed;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.report.ReportTestUtil;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

public class ReportFreePettyCashTest {

	private ReportFreePettyCash make() {
		return new ReportFreePettyCash("team1", 50000);
	}

	@Test
	public void serializationRoundTrip() {
		ReportFreePettyCash original = make();
		JsonObject json = (JsonObject) original.toJsonValue();
		ReportFreePettyCash restored = new ReportFreePettyCash().initFrom(ReportTestUtil.source(), json);
		assertEquals(original.getTeamId(), restored.getTeamId());
		assertEquals(original.getGold(), restored.getGold());
	}

	@Test
	public void toJsonValueHasReportId() {
		JsonObject json = (JsonObject) make().toJsonValue();
		assertEquals("freePettyCash", json.get("reportId").asString());
	}
}
