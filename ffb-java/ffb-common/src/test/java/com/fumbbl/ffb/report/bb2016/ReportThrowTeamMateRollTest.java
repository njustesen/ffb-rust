package com.fumbbl.ffb.report.bb2016;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.PassingDistance;
import com.fumbbl.ffb.modifiers.PassModifier;
import com.fumbbl.ffb.report.ReportTestUtil;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

public class ReportThrowTeamMateRollTest {

	private ReportThrowTeamMateRoll make() {
		return new ReportThrowTeamMateRoll("thrower", true, 4, 3, false, new PassModifier[0], PassingDistance.SHORT_PASS,
			"thrown");
	}

	@Test
	public void serializationRoundTrip() {
		ReportThrowTeamMateRoll original = make();
		JsonObject json = original.toJsonValue();
		ReportThrowTeamMateRoll restored = new ReportThrowTeamMateRoll().initFrom(ReportTestUtil.source(), json);
		assertEquals(original.getPlayerId(), restored.getPlayerId());
		assertEquals(original.isSuccessful(), restored.isSuccessful());
		assertEquals(original.getThrownPlayerId(), restored.getThrownPlayerId());
		assertEquals(original.getPassingDistance(), restored.getPassingDistance());
	}

	@Test
	public void toJsonValueHasReportId() {
		JsonObject json = make().toJsonValue();
		assertEquals("throwTeamMateRoll", json.get("reportId").asString());
	}
}
