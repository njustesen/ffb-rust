package com.fumbbl.ffb.report;

import com.eclipsesource.json.JsonObject;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertArrayEquals;
import static org.junit.jupiter.api.Assertions.assertEquals;

public class ReportJumpUpRollTest {

	private ReportJumpUpRoll make() {
		return new ReportJumpUpRoll("p1", false, 1, 3, true, null);
	}

	@Test
	public void serializationRoundTrip() {
		ReportJumpUpRoll original = make();
		JsonObject json = original.toJsonValue();
		ReportJumpUpRoll restored = (ReportJumpUpRoll) new ReportJumpUpRoll().initFrom(ReportTestUtil.source(), json);
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
		assertEquals("jumpUpRoll", json.get("reportId").asString());
	}
}
