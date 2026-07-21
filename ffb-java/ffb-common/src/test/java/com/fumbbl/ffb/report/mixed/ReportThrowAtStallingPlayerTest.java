package com.fumbbl.ffb.report.mixed;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.report.ReportTestUtil;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

public class ReportThrowAtStallingPlayerTest {

	private ReportThrowAtStallingPlayer make() {
		return new ReportThrowAtStallingPlayer("p1", 5, true);
	}

	@Test
	public void serializationRoundTrip() {
		ReportThrowAtStallingPlayer original = make();
		JsonObject json = original.toJsonValue().asObject();
		ReportThrowAtStallingPlayer restored = new ReportThrowAtStallingPlayer().initFrom(ReportTestUtil.source(), json);
		assertEquals(original.getPlayerId(), restored.getPlayerId());
		assertEquals(original.getRoll(), restored.getRoll());
		assertEquals(original.isSuccessful(), restored.isSuccessful());
	}

	@Test
	public void toJsonValueHasReportId() {
		JsonObject json = make().toJsonValue().asObject();
		assertEquals("throwAtStallingPlayer", json.get("reportId").asString());
	}
}
