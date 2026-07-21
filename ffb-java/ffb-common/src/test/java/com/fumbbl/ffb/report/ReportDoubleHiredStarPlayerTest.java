package com.fumbbl.ffb.report;

import com.eclipsesource.json.JsonObject;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

public class ReportDoubleHiredStarPlayerTest {

	private ReportDoubleHiredStarPlayer make() {
		return new ReportDoubleHiredStarPlayer("Griff Oberwald");
	}

	@Test
	public void serializationRoundTrip() {
		ReportDoubleHiredStarPlayer original = make();
		JsonObject json = original.toJsonValue();
		ReportDoubleHiredStarPlayer restored = new ReportDoubleHiredStarPlayer().initFrom(ReportTestUtil.source(), json);
		assertEquals(original.getStarPlayerName(), restored.getStarPlayerName());
	}

	@Test
	public void toJsonValueHasReportId() {
		JsonObject json = make().toJsonValue();
		assertEquals("doubleHiredStarPlayer", json.get("reportId").asString());
	}
}
