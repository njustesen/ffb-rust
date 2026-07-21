package com.fumbbl.ffb.report;

import com.eclipsesource.json.JsonObject;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertArrayEquals;
import static org.junit.jupiter.api.Assertions.assertEquals;

public class ReportDauntlessRollTest {

	private ReportDauntlessRoll make() {
		return new ReportDauntlessRoll("p1", true, 5, 3, false, 4, "d1");
	}

	@Test
	public void serializationRoundTrip() {
		ReportDauntlessRoll original = make();
		JsonObject json = original.toJsonValue();
		ReportDauntlessRoll restored = new ReportDauntlessRoll().initFrom(ReportTestUtil.source(), json);
		assertEquals(original.getPlayerId(), restored.getPlayerId());
		assertEquals(original.isSuccessful(), restored.isSuccessful());
		assertEquals(original.getRoll(), restored.getRoll());
		assertEquals(original.getMinimumRoll(), restored.getMinimumRoll());
		assertEquals(original.isReRolled(), restored.isReRolled());
		assertArrayEquals(original.getRollModifiers(), restored.getRollModifiers());
		assertEquals(original.getStrength(), restored.getStrength());
		assertEquals(original.getDefenderId(), restored.getDefenderId());
	}

	@Test
	public void toJsonValueHasReportId() {
		JsonObject json = make().toJsonValue();
		assertEquals("dauntlessRoll", json.get("reportId").asString());
	}
}
