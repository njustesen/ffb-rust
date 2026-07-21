package com.fumbbl.ffb.report;

import com.eclipsesource.json.JsonObject;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

public class ReportCoinThrowTest {

	private ReportCoinThrow make() {
		return new ReportCoinThrow(true, "CoachA", false);
	}

	@Test
	public void serializationRoundTrip() {
		ReportCoinThrow original = make();
		JsonObject json = original.toJsonValue();
		ReportCoinThrow restored = new ReportCoinThrow().initFrom(ReportTestUtil.source(), json);
		assertEquals(original.getCoach(), restored.getCoach());
		assertEquals(original.isCoinThrowHeads(), restored.isCoinThrowHeads());
		assertEquals(original.isCoinChoiceHeads(), restored.isCoinChoiceHeads());
	}

	@Test
	public void toJsonValueHasReportId() {
		JsonObject json = make().toJsonValue();
		assertEquals("coinThrow", json.get("reportId").asString());
	}
}
