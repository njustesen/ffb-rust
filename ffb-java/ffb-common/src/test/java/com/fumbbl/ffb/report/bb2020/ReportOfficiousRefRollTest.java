package com.fumbbl.ffb.report.bb2020;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.report.ReportTestUtil;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

public class ReportOfficiousRefRollTest {
	private ReportOfficiousRefRoll make() {
		return new ReportOfficiousRefRoll(4, "p1");
	}

	@Test
	public void serializationRoundTrip() {
		ReportOfficiousRefRoll original = make();
		JsonObject json = original.toJsonValue();
		ReportOfficiousRefRoll restored = new ReportOfficiousRefRoll().initFrom(ReportTestUtil.source(), json);
		assertEquals(original.getRoll(), restored.getRoll());
		assertEquals(original.getPlayerId(), restored.getPlayerId());
	}

	@Test
	public void toJsonValueHasReportId() {
		JsonObject json = make().toJsonValue();
		assertEquals("officiousRefRoll", json.get("reportId").asString());
	}
}
