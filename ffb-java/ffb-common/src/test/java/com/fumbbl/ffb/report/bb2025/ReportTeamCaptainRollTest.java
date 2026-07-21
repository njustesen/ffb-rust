package com.fumbbl.ffb.report.bb2025;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.report.ReportTestUtil;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

public class ReportTeamCaptainRollTest {
	private ReportTeamCaptainRoll make() {
		return new ReportTeamCaptainRoll("team1", 4, 5, true);
	}

	@Test
	public void serializationRoundTrip() {
		ReportTeamCaptainRoll original = make();
		JsonObject json = original.toJsonValue();
		ReportTeamCaptainRoll restored = new ReportTeamCaptainRoll().initFrom(ReportTestUtil.source(), json);
		assertEquals(original.getTeamId(), restored.getTeamId());
		assertEquals(original.getMinimumRoll(), restored.getMinimumRoll());
		assertEquals(original.getRoll(), restored.getRoll());
		assertEquals(original.isSuccessful(), restored.isSuccessful());
	}

	@Test
	public void toJsonValueHasReportId() {
		JsonObject json = make().toJsonValue();
		assertEquals("teamCaptainRoll", json.get("reportId").asString());
	}
}
