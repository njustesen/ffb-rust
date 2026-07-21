package com.fumbbl.ffb.report.bb2020;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.report.ReportTestUtil;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

public class ReportPrayerRollTest {
	private ReportPrayerRoll make() {
		return new ReportPrayerRoll(3);
	}

	@Test
	public void serializationRoundTrip() {
		ReportPrayerRoll original = make();
		JsonObject json = original.toJsonValue();
		ReportPrayerRoll restored = new ReportPrayerRoll().initFrom(ReportTestUtil.source(), json);
		assertEquals(original.getRoll(), restored.getRoll());
	}

	@Test
	public void toJsonValueHasReportId() {
		JsonObject json = make().toJsonValue();
		assertEquals("prayerRoll", json.get("reportId").asString());
	}
}
