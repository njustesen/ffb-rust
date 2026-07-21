package com.fumbbl.ffb.report;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.modifiers.RollModifier;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertArrayEquals;
import static org.junit.jupiter.api.Assertions.assertEquals;

public class ReportSafeThrowRollTest {

	private ReportSafeThrowRoll make() {
		return new ReportSafeThrowRoll("p1", true, 3, 2, false, new RollModifier[0]);
	}

	@Test
	public void serializationRoundTrip() {
		ReportSafeThrowRoll original = make();
		JsonObject json = original.toJsonValue();
		ReportSafeThrowRoll restored = (ReportSafeThrowRoll) new ReportSafeThrowRoll().initFrom(ReportTestUtil.source(), json);
		assertEquals(original.getPlayerId(), restored.getPlayerId());
		assertEquals(original.isSuccessful(), restored.isSuccessful());
		assertEquals(original.getRoll(), restored.getRoll());
		assertEquals(original.getMinimumRoll(), restored.getMinimumRoll());
		assertEquals(original.isReRolled(), restored.isReRolled());
		assertArrayEquals(original.getRollModifiers(), restored.getRollModifiers());
	}

	@Test
	public void toJsonValueHasReportId() {
		JsonObject json = make().toJsonValue();
		assertEquals("safeThrowRoll", json.get("reportId").asString());
	}
}
