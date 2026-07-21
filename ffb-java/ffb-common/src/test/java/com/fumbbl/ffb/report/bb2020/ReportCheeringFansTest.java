package com.fumbbl.ffb.report.bb2020;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.report.ReportTestUtil;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

public class ReportCheeringFansTest {
	private ReportCheeringFans make() {
		return new ReportCheeringFans("team1", true, 4, 2);
	}

	@Test
	public void serializationRoundTrip() {
		ReportCheeringFans original = make();
		JsonObject json = original.toJsonValue();
		ReportCheeringFans restored = new ReportCheeringFans().initFrom(ReportTestUtil.source(), json);
		assertEquals(original.getTeamId(), restored.getTeamId());
		assertEquals(original.isPrayerAvailable(), restored.isPrayerAvailable());
		assertEquals(original.getRollHome(), restored.getRollHome());
		assertEquals(original.getRollAway(), restored.getRollAway());
	}

	@Test
	public void toJsonValueHasReportId() {
		JsonObject json = make().toJsonValue();
		assertEquals("cheeringFans", json.get("reportId").asString());
	}
}
