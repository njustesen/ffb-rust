package com.fumbbl.ffb.report.mixed;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.report.ReportTestUtil;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

public class ReportPenaltyShootoutTest {

	private ReportPenaltyShootout make() {
		return new ReportPenaltyShootout(4, 1, 3, 0, true, "1", "home");
	}

	@Test
	public void serializationRoundTrip() {
		ReportPenaltyShootout original = make();
		JsonObject json = original.toJsonValue().asObject();
		ReportPenaltyShootout restored = new ReportPenaltyShootout().initFrom(ReportTestUtil.source(), json);
		assertEquals(original.getRollHome(), restored.getRollHome());
		assertEquals(original.getRollAway(), restored.getRollAway());
		assertEquals(original.getScoreHome(), restored.getScoreHome());
		assertEquals(original.getScoreAway(), restored.getScoreAway());
		assertEquals(original.getRollCount(), restored.getRollCount());
		assertEquals(original.getWinningTeam(), restored.getWinningTeam());
		assertEquals(original.getHomeTeamWonPenalty(), restored.getHomeTeamWonPenalty());
	}

	@Test
	public void toJsonValueHasReportId() {
		JsonObject json = make().toJsonValue().asObject();
		assertEquals("penaltyShootout", json.get("reportId").asString());
	}
}
