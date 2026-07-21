package com.fumbbl.ffb.report.bb2016;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.report.ReportTestUtil;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

public class ReportNoPlayersToFieldTest {

	private ReportNoPlayersToField make() {
		return new ReportNoPlayersToField("team1");
	}

	@Test
	public void serializationRoundTrip() {
		ReportNoPlayersToField original = make();
		JsonObject json = original.toJsonValue();
		ReportNoPlayersToField restored = new ReportNoPlayersToField().initFrom(ReportTestUtil.source(), json);
		assertEquals(original.getTeamId(), restored.getTeamId());
	}

	@Test
	public void toJsonValueHasReportId() {
		JsonObject json = make().toJsonValue();
		assertEquals("noPlayersToField", json.get("reportId").asString());
	}
}
