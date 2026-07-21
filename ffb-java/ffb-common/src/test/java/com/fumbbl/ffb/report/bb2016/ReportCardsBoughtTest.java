package com.fumbbl.ffb.report.bb2016;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.report.ReportTestUtil;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

public class ReportCardsBoughtTest {

	private ReportCardsBought make() {
		return new ReportCardsBought("team1", 2, 50000);
	}

	@Test
	public void serializationRoundTrip() {
		ReportCardsBought original = make();
		JsonObject json = original.toJsonValue();
		ReportCardsBought restored = new ReportCardsBought().initFrom(ReportTestUtil.source(), json);
		assertEquals(original.getTeamId(), restored.getTeamId());
		assertEquals(original.getNrOfCards(), restored.getNrOfCards());
		assertEquals(original.getGold(), restored.getGold());
	}

	@Test
	public void toJsonValueHasReportId() {
		JsonObject json = make().toJsonValue();
		assertEquals("cardsBought", json.get("reportId").asString());
	}
}
