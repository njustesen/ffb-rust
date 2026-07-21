package com.fumbbl.ffb.report.mixed;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.report.ReportTestUtil;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

public class ReportKickoffExtraReRollTest {

	private ReportKickoffExtraReRoll make() {
		return new ReportKickoffExtraReRoll(4, 2, "team1");
	}

	@Test
	public void serializationRoundTrip() {
		ReportKickoffExtraReRoll original = make();
		JsonObject json = (JsonObject) original.toJsonValue();
		ReportKickoffExtraReRoll restored = new ReportKickoffExtraReRoll().initFrom(ReportTestUtil.source(), json);
		assertEquals(original.getRollHome(), restored.getRollHome());
		assertEquals(original.getRollAway(), restored.getRollAway());
		assertEquals(original.getTeamId(), restored.getTeamId());
	}

	@Test
	public void toJsonValueHasReportId() {
		JsonObject json = (JsonObject) make().toJsonValue();
		assertEquals("extraReRoll", json.get("reportId").asString());
	}
}
