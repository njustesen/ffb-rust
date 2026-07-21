package com.fumbbl.ffb.report.bb2020;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.report.ReportTestUtil;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

public class ReportCardsAndInducementsBoughtTest {
	private ReportCardsAndInducementsBought make() {
		return new ReportCardsAndInducementsBought("team1", 2, 1, 0, 0, 100000, 1200000);
	}

	@Test
	public void serializationRoundTrip() {
		ReportCardsAndInducementsBought original = make();
		JsonObject json = original.toJsonValue();
		ReportCardsAndInducementsBought restored = new ReportCardsAndInducementsBought().initFrom(ReportTestUtil.source(), json);
		assertEquals(original.getTeamId(), restored.getTeamId());
		assertEquals(original.getCards(), restored.getCards());
		assertEquals(original.getInducements(), restored.getInducements());
		assertEquals(original.getStars(), restored.getStars());
		assertEquals(original.getMercenaries(), restored.getMercenaries());
		assertEquals(original.getGold(), restored.getGold());
		assertEquals(original.getNewTv(), restored.getNewTv());
	}

	@Test
	public void toJsonValueHasReportId() {
		JsonObject json = make().toJsonValue();
		assertEquals("cardsAndInducementsBought", json.get("reportId").asString());
	}
}
