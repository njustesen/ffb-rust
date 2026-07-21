package com.fumbbl.ffb.report.bb2025;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.report.ReportTestUtil;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

public class ReportPrayerRollTest {
	private ReportPrayerRoll make() {
		return new ReportPrayerRoll("Home Ultras", 5, true);
	}

	@Test
	public void serializationRoundTrip() {
		ReportPrayerRoll original = make();
		JsonObject json = original.toJsonValue();
		ReportPrayerRoll restored = new ReportPrayerRoll().initFrom(ReportTestUtil.source(), json);
		assertEquals(original.getTeamName(), restored.getTeamName());
		assertEquals(original.getRoll(), restored.getRoll());
		assertEquals(original.isHomeTeam(), restored.isHomeTeam());
	}

	@Test
	public void toJsonValueHasReportId() {
		JsonObject json = make().toJsonValue();
		assertEquals("prayerRoll", json.get("reportId").asString());
	}
}
