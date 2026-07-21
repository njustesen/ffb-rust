package com.fumbbl.ffb.report.bb2025;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.report.ReportTestUtil;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

public class ReportPrayersAndInducementsBoughtTest {
	private ReportPrayersAndInducementsBought make() {
		return new ReportPrayersAndInducementsBought("team1", 2, 1, 0, 150000, 1100000);
	}

	@Test
	public void serializationRoundTrip() {
		ReportPrayersAndInducementsBought original = make();
		JsonObject json = original.toJsonValue();
		ReportPrayersAndInducementsBought restored = new ReportPrayersAndInducementsBought().initFrom(ReportTestUtil.source(), json);
		assertEquals(original.getTeamId(), restored.getTeamId());
		assertEquals(original.getInducements(), restored.getInducements());
		assertEquals(original.getStars(), restored.getStars());
		assertEquals(original.getMercenaries(), restored.getMercenaries());
		assertEquals(original.getGold(), restored.getGold());
		assertEquals(original.getNewTv(), restored.getNewTv());
	}

	@Test
	public void toJsonValueHasReportId() {
		JsonObject json = make().toJsonValue();
		assertEquals("prayersAndInducementsBought", json.get("reportId").asString());
	}
}
