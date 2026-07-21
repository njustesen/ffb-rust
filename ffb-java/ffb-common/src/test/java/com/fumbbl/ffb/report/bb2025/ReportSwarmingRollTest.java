package com.fumbbl.ffb.report.bb2025;

import com.eclipsesource.json.JsonValue;
import com.fumbbl.ffb.report.ReportTestUtil;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

public class ReportSwarmingRollTest {
	private ReportSwarmingRoll make() {
		return new ReportSwarmingRoll("t1", 2);
	}

	@Test
	public void serializationRoundTrip() {
		ReportSwarmingRoll original = make();
		JsonValue json = original.toJsonValue();
		ReportSwarmingRoll restored = (ReportSwarmingRoll) new ReportSwarmingRoll().initFrom(ReportTestUtil.source(), json);
		assertEquals(original.getTeamId(), restored.getTeamId());
		assertEquals(original.getRoll(), restored.getRoll());
	}

	@Test
	public void toJsonValueHasReportId() {
		JsonValue json = make().toJsonValue();
		assertEquals("swarmingPlayersRoll", json.asObject().get("reportId").asString());
	}
}
