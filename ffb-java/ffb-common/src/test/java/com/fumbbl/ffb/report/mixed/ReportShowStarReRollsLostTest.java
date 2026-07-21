package com.fumbbl.ffb.report.mixed;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.report.ReportTestUtil;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

public class ReportShowStarReRollsLostTest {

	private ReportShowStarReRollsLost make() {
		return new ReportShowStarReRollsLost("team1", 1);
	}

	@Test
	public void serializationRoundTrip() {
		ReportShowStarReRollsLost original = make();
		JsonObject json = original.toJsonValue().asObject();
		ReportShowStarReRollsLost restored =
			(ReportShowStarReRollsLost) new ReportShowStarReRollsLost().initFrom(ReportTestUtil.source(), json);
		assertEquals(original.getTeamId(), restored.getTeamId());
		assertEquals(original.getAmount(), restored.getAmount());
	}

	@Test
	public void toJsonValueHasReportId() {
		JsonObject json = make().toJsonValue().asObject();
		assertEquals("showStarReRollLost", json.get("reportId").asString());
	}
}
