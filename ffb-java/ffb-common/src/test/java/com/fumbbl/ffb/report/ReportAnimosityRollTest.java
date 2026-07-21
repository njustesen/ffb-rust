package com.fumbbl.ffb.report;

import com.eclipsesource.json.JsonObject;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertArrayEquals;
import static org.junit.jupiter.api.Assertions.assertEquals;

public class ReportAnimosityRollTest {

	private ReportAnimosityRoll make() {
		return new ReportAnimosityRoll("p1", false, 2, 3, true, null);
	}

	@Test
	public void serializationRoundTrip() {
		ReportAnimosityRoll original = make();
		JsonObject json = original.toJsonValue();
		ReportAnimosityRoll restored = (ReportAnimosityRoll) new ReportAnimosityRoll().initFrom(ReportTestUtil.source(), json);
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
		assertEquals("animosityRoll", json.get("reportId").asString());
	}
}
