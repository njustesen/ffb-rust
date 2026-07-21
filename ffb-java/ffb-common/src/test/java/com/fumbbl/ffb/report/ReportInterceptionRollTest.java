package com.fumbbl.ffb.report;

import com.eclipsesource.json.JsonObject;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertArrayEquals;
import static org.junit.jupiter.api.Assertions.assertEquals;

public class ReportInterceptionRollTest {

	private ReportInterceptionRoll make() {
		return new ReportInterceptionRoll("p1", false, 3, 5, false, null, true, false);
	}

	@Test
	public void serializationRoundTrip() {
		ReportInterceptionRoll original = make();
		JsonObject json = original.toJsonValue();
		ReportInterceptionRoll restored = new ReportInterceptionRoll().initFrom(ReportTestUtil.source(), json);
		assertEquals(original.getPlayerId(), restored.getPlayerId());
		assertEquals(original.isSuccessful(), restored.isSuccessful());
		assertEquals(original.getRoll(), restored.getRoll());
		assertEquals(original.getMinimumRoll(), restored.getMinimumRoll());
		assertEquals(original.isReRolled(), restored.isReRolled());
		assertArrayEquals(original.getRollModifiers(), restored.getRollModifiers());
		assertEquals(original.isBomb(), restored.isBomb());
		assertEquals(original.isIgnoreAgility(), restored.isIgnoreAgility());
	}

	@Test
	public void toJsonValueHasReportId() {
		JsonObject json = make().toJsonValue();
		assertEquals("interceptionRoll", json.get("reportId").asString());
	}
}
