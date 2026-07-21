package com.fumbbl.ffb.report;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.modifiers.RollModifier;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertArrayEquals;
import static org.junit.jupiter.api.Assertions.assertEquals;

public class ReportRegenerationRollTest {

	private ReportRegenerationRoll make() {
		return new ReportRegenerationRoll("p1", true, 4, 4, false, new RollModifier[0]);
	}

	@Test
	public void serializationRoundTrip() {
		ReportRegenerationRoll original = make();
		JsonObject json = original.toJsonValue();
		ReportRegenerationRoll restored = (ReportRegenerationRoll) new ReportRegenerationRoll().initFrom(ReportTestUtil.source(), json);
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
		assertEquals("regenerationRoll", json.get("reportId").asString());
	}
}
