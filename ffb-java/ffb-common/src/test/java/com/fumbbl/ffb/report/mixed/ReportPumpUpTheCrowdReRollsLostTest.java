package com.fumbbl.ffb.report.mixed;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.report.ReportTestUtil;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

public class ReportPumpUpTheCrowdReRollsLostTest {

	private ReportPumpUpTheCrowdReRollsLost make() {
		return new ReportPumpUpTheCrowdReRollsLost("team1", 2);
	}

	@Test
	public void serializationRoundTrip() {
		ReportPumpUpTheCrowdReRollsLost original = make();
		JsonObject json = original.toJsonValue().asObject();
		ReportPumpUpTheCrowdReRollsLost restored =
			(ReportPumpUpTheCrowdReRollsLost) new ReportPumpUpTheCrowdReRollsLost().initFrom(ReportTestUtil.source(), json);
		assertEquals(original.getTeamId(), restored.getTeamId());
		assertEquals(original.getAmount(), restored.getAmount());
	}

	@Test
	public void toJsonValueHasReportId() {
		JsonObject json = make().toJsonValue().asObject();
		assertEquals("pumpUpTheCrowdReRollLost", json.get("reportId").asString());
	}
}
