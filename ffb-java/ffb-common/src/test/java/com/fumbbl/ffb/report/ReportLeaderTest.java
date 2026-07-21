package com.fumbbl.ffb.report;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.LeaderState;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

public class ReportLeaderTest {

	private ReportLeader make() {
		return new ReportLeader("team1", LeaderState.AVAILABLE);
	}

	@Test
	public void serializationRoundTrip() {
		ReportLeader original = make();
		JsonObject json = original.toJsonValue();
		ReportLeader restored = new ReportLeader().initFrom(ReportTestUtil.source(), json);
		assertEquals(original.getTeamId(), restored.getTeamId());
		assertEquals(original.getLeaderState(), restored.getLeaderState());
	}

	@Test
	public void toJsonValueHasReportId() {
		JsonObject json = make().toJsonValue();
		assertEquals("leader", json.get("reportId").asString());
	}
}
