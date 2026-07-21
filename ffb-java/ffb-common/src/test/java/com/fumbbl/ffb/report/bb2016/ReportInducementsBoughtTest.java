package com.fumbbl.ffb.report.bb2016;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.report.ReportTestUtil;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

public class ReportInducementsBoughtTest {

	private ReportInducementsBought make() {
		return new ReportInducementsBought("team1", 3, 1, 0, 150000);
	}

	@Test
	public void serializationRoundTrip() {
		ReportInducementsBought original = make();
		JsonObject json = original.toJsonValue();
		ReportInducementsBought restored = new ReportInducementsBought().initFrom(ReportTestUtil.source(), json);
		assertEquals(original.getTeamId(), restored.getTeamId());
		assertEquals(original.getNrOfInducements(), restored.getNrOfInducements());
		assertEquals(original.getNrOfStars(), restored.getNrOfStars());
		assertEquals(original.getNrOfMercenaries(), restored.getNrOfMercenaries());
		assertEquals(original.getGold(), restored.getGold());
	}

	@Test
	public void toJsonValueHasReportId() {
		JsonObject json = make().toJsonValue();
		assertEquals("inducementsBought", json.get("reportId").asString());
	}
}
