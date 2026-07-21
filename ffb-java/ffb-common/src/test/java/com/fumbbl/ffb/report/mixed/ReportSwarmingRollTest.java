package com.fumbbl.ffb.report.mixed;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.report.ReportTestUtil;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

public class ReportSwarmingRollTest {

	private ReportSwarmingRoll make() {
		return new ReportSwarmingRoll("team1", 3, 2, 4);
	}

	@Test
	public void serializationRoundTrip() {
		ReportSwarmingRoll original = make();
		JsonObject json = original.toJsonValue().asObject();
		ReportSwarmingRoll restored =
			(ReportSwarmingRoll) new ReportSwarmingRoll().initFrom(ReportTestUtil.source(), json);
		assertEquals(original.getTeamId(), restored.getTeamId());
		assertEquals(original.getAmount(), restored.getAmount());
		assertEquals(original.getRoll(), restored.getRoll());
		assertEquals(original.getLimit(), restored.getLimit());
	}

	@Test
	public void toJsonValueHasReportId() {
		JsonObject json = make().toJsonValue().asObject();
		assertEquals("swarmingPlayersRoll", json.get("reportId").asString());
	}
}
