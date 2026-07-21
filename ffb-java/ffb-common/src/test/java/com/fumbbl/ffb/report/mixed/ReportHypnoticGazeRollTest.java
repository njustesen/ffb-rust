package com.fumbbl.ffb.report.mixed;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.modifiers.RollModifier;
import com.fumbbl.ffb.report.ReportTestUtil;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

public class ReportHypnoticGazeRollTest {

	private ReportHypnoticGazeRoll make() {
		return new ReportHypnoticGazeRoll("p1", true, 4, 2, false, new RollModifier[0], "d1");
	}

	@Test
	public void serializationRoundTrip() {
		ReportHypnoticGazeRoll original = make();
		JsonObject json = (JsonObject) original.toJsonValue();
		ReportHypnoticGazeRoll restored = (ReportHypnoticGazeRoll) new ReportHypnoticGazeRoll().initFrom(ReportTestUtil.source(), json);
		assertEquals(original.getPlayerId(), restored.getPlayerId());
		assertEquals(original.isSuccessful(), restored.isSuccessful());
		assertEquals(original.getRoll(), restored.getRoll());
		assertEquals(original.getMinimumRoll(), restored.getMinimumRoll());
		assertEquals(original.isReRolled(), restored.isReRolled());
		assertEquals(original.getDefenderId(), restored.getDefenderId());
	}

	@Test
	public void toJsonValueHasReportId() {
		JsonObject json = (JsonObject) make().toJsonValue();
		assertEquals("hypnoticGazeRoll", json.get("reportId").asString());
	}
}
