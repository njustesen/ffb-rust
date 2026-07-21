package com.fumbbl.ffb.report.mixed;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.BreatheFireResult;
import com.fumbbl.ffb.report.ReportTestUtil;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

public class ReportBreatheFireTest {

	private ReportBreatheFire make() {
		return new ReportBreatheFire("p1", true, 4, 2, false, "d1", BreatheFireResult.KNOCK_DOWN, false);
	}

	@Test
	public void serializationRoundTrip() {
		ReportBreatheFire original = make();
		JsonObject json = (JsonObject) original.toJsonValue();
		ReportBreatheFire restored = new ReportBreatheFire().initFrom(ReportTestUtil.source(), json);
		assertEquals(original.getPlayerId(), restored.getPlayerId());
		assertEquals(original.isSuccessful(), restored.isSuccessful());
		assertEquals(original.getRoll(), restored.getRoll());
		assertEquals(original.getMinimumRoll(), restored.getMinimumRoll());
		assertEquals(original.isReRolled(), restored.isReRolled());
		assertEquals(original.getDefenderId(), restored.getDefenderId());
		assertEquals(original.isStrongOpponent(), restored.isStrongOpponent());
		assertEquals(original.getResult(), restored.getResult());
	}

	@Test
	public void toJsonValueHasReportId() {
		JsonObject json = (JsonObject) make().toJsonValue();
		assertEquals("breatheFire", json.get("reportId").asString());
	}
}
