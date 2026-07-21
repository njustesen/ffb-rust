package com.fumbbl.ffb.report;

import com.eclipsesource.json.JsonObject;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertArrayEquals;
import static org.junit.jupiter.api.Assertions.assertEquals;

public class ReportJumpRollTest {

	private ReportJumpRoll make() {
		return new ReportJumpRoll("p1", true, 4, 3, false, null);
	}

	@Test
	public void serializationRoundTrip() {
		ReportJumpRoll original = make();
		JsonObject json = original.toJsonValue();
		ReportJumpRoll restored = (ReportJumpRoll) new ReportJumpRoll().initFrom(ReportTestUtil.source(), json);
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
		assertEquals("leapRoll", json.get("reportId").asString());
	}
}
