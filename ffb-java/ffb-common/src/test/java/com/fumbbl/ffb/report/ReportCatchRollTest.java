package com.fumbbl.ffb.report;

import com.eclipsesource.json.JsonObject;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertArrayEquals;
import static org.junit.jupiter.api.Assertions.assertEquals;

public class ReportCatchRollTest {

	private ReportCatchRoll make() {
		return new ReportCatchRoll("p1", true, 4, 3, false, null, false);
	}

	@Test
	public void serializationRoundTrip() {
		ReportCatchRoll original = make();
		JsonObject json = original.toJsonValue();
		ReportCatchRoll restored = new ReportCatchRoll().initFrom(ReportTestUtil.source(), json);
		assertEquals(original.getPlayerId(), restored.getPlayerId());
		assertEquals(original.isSuccessful(), restored.isSuccessful());
		assertEquals(original.getRoll(), restored.getRoll());
		assertEquals(original.getMinimumRoll(), restored.getMinimumRoll());
		assertEquals(original.isReRolled(), restored.isReRolled());
		assertArrayEquals(original.getRollModifiers(), restored.getRollModifiers());
		assertEquals(original.isBomb(), restored.isBomb());
	}

	@Test
	public void toJsonValueHasReportId() {
		JsonObject json = make().toJsonValue();
		assertEquals("catchRoll", json.get("reportId").asString());
	}
}
