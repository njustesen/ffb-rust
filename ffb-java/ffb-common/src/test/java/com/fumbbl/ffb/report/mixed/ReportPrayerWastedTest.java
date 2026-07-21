package com.fumbbl.ffb.report.mixed;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.report.ReportTestUtil;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

public class ReportPrayerWastedTest {

	private ReportPrayerWasted make() {
		return new ReportPrayerWasted("PRAYER_OF_DEATH", "p1");
	}

	@Test
	public void serializationRoundTrip() {
		ReportPrayerWasted original = make();
		JsonObject json = original.toJsonValue().asObject();
		ReportPrayerWasted restored = new ReportPrayerWasted().initFrom(ReportTestUtil.source(), json);
		assertEquals(original.getPrayerName(), restored.getPrayerName());
		assertEquals(original.getPlayerId(), restored.getPlayerId());
	}

	@Test
	public void toJsonValueHasReportId() {
		JsonObject json = make().toJsonValue().asObject();
		assertEquals("prayerWasted", json.get("reportId").asString());
	}
}
