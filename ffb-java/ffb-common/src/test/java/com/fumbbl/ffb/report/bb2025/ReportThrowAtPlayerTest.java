package com.fumbbl.ffb.report.bb2025;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.report.ReportTestUtil;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

public class ReportThrowAtPlayerTest {
	private ReportThrowAtPlayer make() {
		return new ReportThrowAtPlayer("p1", 4, true);
	}

	@Test
	public void serializationRoundTrip() {
		ReportThrowAtPlayer original = make();
		JsonObject json = original.toJsonValue();
		ReportThrowAtPlayer restored = new ReportThrowAtPlayer().initFrom(ReportTestUtil.source(), json);
		assertEquals(original.getPlayerId(), restored.getPlayerId());
		assertEquals(original.getRoll(), restored.getRoll());
		assertEquals(original.isSuccessful(), restored.isSuccessful());
	}

	@Test
	public void toJsonValueHasReportId() {
		JsonObject json = make().toJsonValue();
		assertEquals("throwAtPlayer", json.get("reportId").asString());
	}
}
