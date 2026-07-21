package com.fumbbl.ffb.report.mixed;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.report.ReportTestUtil;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

public class ReportBrilliantCoachingReRollsLostTest {

	private ReportBrilliantCoachingReRollsLost make() {
		return new ReportBrilliantCoachingReRollsLost("team1", 2);
	}

	@Test
	public void serializationRoundTrip() {
		ReportBrilliantCoachingReRollsLost original = make();
		JsonObject json = (JsonObject) original.toJsonValue();
		ReportBrilliantCoachingReRollsLost restored = (ReportBrilliantCoachingReRollsLost) new ReportBrilliantCoachingReRollsLost().initFrom(ReportTestUtil.source(), json);
		assertEquals(original.getTeamId(), restored.getTeamId());
		assertEquals(original.getAmount(), restored.getAmount());
	}

	@Test
	public void toJsonValueHasReportId() {
		JsonObject json = (JsonObject) make().toJsonValue();
		assertEquals("brilliantCoachingReRoll", json.get("reportId").asString());
	}
}
