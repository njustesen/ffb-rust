package com.fumbbl.ffb.report;

import com.eclipsesource.json.JsonObject;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertArrayEquals;
import static org.junit.jupiter.api.Assertions.assertEquals;

public class ReportMostValuablePlayersTest {

	private ReportMostValuablePlayers make() {
		ReportMostValuablePlayers report = new ReportMostValuablePlayers();
		report.addPlayerIdHome("h1");
		report.addPlayerIdHome("h2");
		report.addPlayerIdAway("a1");
		return report;
	}

	@Test
	public void serializationRoundTrip() {
		ReportMostValuablePlayers original = make();
		JsonObject json = original.toJsonValue();
		ReportMostValuablePlayers restored = new ReportMostValuablePlayers().initFrom(ReportTestUtil.source(), json);
		assertArrayEquals(original.getPlayerIdsHome(), restored.getPlayerIdsHome());
		assertArrayEquals(original.getPlayerIdsAway(), restored.getPlayerIdsAway());
	}

	@Test
	public void toJsonValueHasReportId() {
		JsonObject json = make().toJsonValue();
		assertEquals("mostValuablePlayers", json.get("reportId").asString());
	}
}
