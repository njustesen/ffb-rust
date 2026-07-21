package com.fumbbl.ffb.report;

import com.eclipsesource.json.JsonObject;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

public class ReportStandUpRollTest {

	private ReportStandUpRoll make() {
		return new ReportStandUpRoll("p1", true, 4, 1, false);
	}

	@Test
	public void serializationRoundTrip() {
		ReportStandUpRoll original = make();
		JsonObject json = original.toJsonValue();
		ReportStandUpRoll restored = new ReportStandUpRoll().initFrom(ReportTestUtil.source(), json);
		assertEquals(original.getPlayerId(), restored.getPlayerId());
		assertEquals(original.isSuccessful(), restored.isSuccessful());
		assertEquals(original.getRoll(), restored.getRoll());
		assertEquals(original.getModifier(), restored.getModifier());
		assertEquals(original.isReRolled(), restored.isReRolled());
	}

	@Test
	public void toJsonValueHasReportId() {
		JsonObject json = make().toJsonValue();
		assertEquals("standUpRoll", json.get("reportId").asString());
	}
}
