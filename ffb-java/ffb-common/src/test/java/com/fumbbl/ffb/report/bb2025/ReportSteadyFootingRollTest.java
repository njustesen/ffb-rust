package com.fumbbl.ffb.report.bb2025;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.report.ReportTestUtil;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

public class ReportSteadyFootingRollTest {
	private ReportSteadyFootingRoll make() {
		return new ReportSteadyFootingRoll("p1", false, 2, 4, true);
	}

	@Test
	public void serializationRoundTrip() {
		ReportSteadyFootingRoll original = make();
		JsonObject json = original.toJsonValue();
		ReportSteadyFootingRoll restored = (ReportSteadyFootingRoll) new ReportSteadyFootingRoll().initFrom(ReportTestUtil.source(), json);
		assertEquals(original.getPlayerId(), restored.getPlayerId());
		assertEquals(original.isSuccessful(), restored.isSuccessful());
		assertEquals(original.getRoll(), restored.getRoll());
		assertEquals(original.getMinimumRoll(), restored.getMinimumRoll());
		assertEquals(original.isReRolled(), restored.isReRolled());
	}

	@Test
	public void toJsonValueHasReportId() {
		JsonObject json = make().toJsonValue();
		assertEquals("steadyFootingRoll", json.get("reportId").asString());
	}
}
