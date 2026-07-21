package com.fumbbl.ffb.report.bb2025;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.report.ReportTestUtil;
import org.junit.jupiter.api.Test;

import java.util.HashSet;
import java.util.Set;

import static org.junit.jupiter.api.Assertions.assertEquals;

public class ReportCheeringFansTest {
	private ReportCheeringFans make() {
		Set<String> teamIds = new HashSet<>();
		teamIds.add("team1");
		Set<String> rerolled = new HashSet<>();
		return new ReportCheeringFans(teamIds, 4, 2, rerolled);
	}

	@Test
	public void serializationRoundTrip() {
		ReportCheeringFans original = make();
		JsonObject json = original.toJsonValue();
		ReportCheeringFans restored = new ReportCheeringFans().initFrom(ReportTestUtil.source(), json);
		assertEquals(original.getTeamIds(), restored.getTeamIds());
		assertEquals(original.getRollHome(), restored.getRollHome());
		assertEquals(original.getRollAway(), restored.getRollAway());
		assertEquals(original.getRerolled(), restored.getRerolled());
	}

	@Test
	public void toJsonValueHasReportId() {
		JsonObject json = make().toJsonValue();
		assertEquals("cheeringFans", json.get("reportId").asString());
	}
}
