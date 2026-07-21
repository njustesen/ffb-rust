package com.fumbbl.ffb.report.mixed;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.report.ReportTestUtil;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

public class ReportKickTeamMateFumbleTest {

	private ReportKickTeamMateFumble make() {
		return new ReportKickTeamMateFumble();
	}

	@Test
	public void serializationRoundTrip() {
		ReportKickTeamMateFumble original = make();
		JsonObject json = (JsonObject) original.toJsonValue();
		ReportKickTeamMateFumble restored = new ReportKickTeamMateFumble().initFrom(ReportTestUtil.source(), json);
		assertEquals(original.getId(), restored.getId());
	}

	@Test
	public void toJsonValueHasReportId() {
		JsonObject json = (JsonObject) make().toJsonValue();
		assertEquals("kickTeamMateFumble", json.get("reportId").asString());
	}
}
