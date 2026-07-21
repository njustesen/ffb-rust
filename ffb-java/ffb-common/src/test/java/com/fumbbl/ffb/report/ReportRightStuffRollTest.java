package com.fumbbl.ffb.report;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.modifiers.RollModifier;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertArrayEquals;
import static org.junit.jupiter.api.Assertions.assertEquals;

public class ReportRightStuffRollTest {

	private ReportRightStuffRoll make() {
		return new ReportRightStuffRoll("p1", false, 2, 4, false, new RollModifier[0]);
	}

	@Test
	public void serializationRoundTrip() {
		ReportRightStuffRoll original = make();
		JsonObject json = original.toJsonValue();
		ReportRightStuffRoll restored = (ReportRightStuffRoll) new ReportRightStuffRoll().initFrom(ReportTestUtil.source(), json);
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
		assertEquals("rightStuffRoll", json.get("reportId").asString());
	}
}
