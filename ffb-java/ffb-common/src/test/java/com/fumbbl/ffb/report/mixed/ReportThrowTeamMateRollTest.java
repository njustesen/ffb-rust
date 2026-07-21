package com.fumbbl.ffb.report.mixed;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.PassingDistance;
import com.fumbbl.ffb.mechanics.PassResult;
import com.fumbbl.ffb.modifiers.PassModifier;
import com.fumbbl.ffb.report.ReportTestUtil;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

public class ReportThrowTeamMateRollTest {

	private ReportThrowTeamMateRoll make() {
		return new ReportThrowTeamMateRoll("thrower", true, 4, 3, false, new PassModifier[0],
			PassingDistance.SHORT_PASS, "thrown", PassResult.ACCURATE, false);
	}

	@Test
	public void serializationRoundTrip() {
		ReportThrowTeamMateRoll original = make();
		JsonObject json = original.toJsonValue().asObject();
		ReportThrowTeamMateRoll restored = new ReportThrowTeamMateRoll().initFrom(ReportTestUtil.source(), json);
		assertEquals(original.getPlayerId(), restored.getPlayerId());
		assertEquals(original.isSuccessful(), restored.isSuccessful());
		assertEquals(original.getRoll(), restored.getRoll());
		assertEquals(original.getMinimumRoll(), restored.getMinimumRoll());
		assertEquals(original.isReRolled(), restored.isReRolled());
		assertEquals(original.getThrownPlayerId(), restored.getThrownPlayerId());
		assertEquals(original.getPassingDistance(), restored.getPassingDistance());
		assertEquals(original.getPassResult(), restored.getPassResult());
		assertEquals(original.isKick(), restored.isKick());
	}

	@Test
	public void toJsonValueHasReportId() {
		JsonObject json = make().toJsonValue().asObject();
		assertEquals("throwTeamMateRoll", json.get("reportId").asString());
	}
}
