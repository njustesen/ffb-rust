package com.fumbbl.ffb.report.bb2016;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.report.ReportTestUtil;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

public class ReportPenaltyShootoutTest {

	private ReportPenaltyShootout make() {
		return new ReportPenaltyShootout(4, 2, 3, 1);
	}

	@Test
	public void serializationRoundTrip() {
		ReportPenaltyShootout original = make();
		JsonObject json = original.toJsonValue();
		ReportPenaltyShootout restored = new ReportPenaltyShootout().initFrom(ReportTestUtil.source(), json);
		assertEquals(original.getRollHome(), restored.getRollHome());
		assertEquals(original.getReRollsLeftHome(), restored.getReRollsLeftHome());
		assertEquals(original.getRollAway(), restored.getRollAway());
		assertEquals(original.getReRollsLeftAway(), restored.getReRollsLeftAway());
	}

	@Test
	public void toJsonValueHasReportId() {
		JsonObject json = make().toJsonValue();
		assertEquals("penaltyShootout", json.get("reportId").asString());
	}
}
