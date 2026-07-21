package com.fumbbl.ffb.report.mixed;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.report.ReportTestUtil;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

public class ReportPrayerAmountTest {

	private ReportPrayerAmount make() {
		return new ReportPrayerAmount(1000, 800, 3, true);
	}

	@Test
	public void serializationRoundTrip() {
		ReportPrayerAmount original = make();
		JsonObject json = original.toJsonValue().asObject();
		ReportPrayerAmount restored = new ReportPrayerAmount().initFrom(ReportTestUtil.source(), json);
		assertEquals(original.getTvHome(), restored.getTvHome());
		assertEquals(original.getTvAway(), restored.getTvAway());
		assertEquals(original.getPrayerAmount(), restored.getPrayerAmount());
		assertEquals(original.isHomeTeamReceivesPrayers(), restored.isHomeTeamReceivesPrayers());
	}

	@Test
	public void toJsonValueHasReportId() {
		JsonObject json = make().toJsonValue().asObject();
		assertEquals("prayerAmount", json.get("reportId").asString());
	}
}
