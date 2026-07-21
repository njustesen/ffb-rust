package com.fumbbl.ffb.report.bb2016;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.report.ReportTestUtil;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertArrayEquals;
import static org.junit.jupiter.api.Assertions.assertEquals;

public class ReportSpectatorsTest {

	private ReportSpectators make() {
		return new ReportSpectators(new int[]{3, 4}, 35000, 1, new int[]{2, 5}, 20000, 0);
	}

	@Test
	public void serializationRoundTrip() {
		ReportSpectators original = make();
		JsonObject json = original.toJsonValue();
		ReportSpectators restored = new ReportSpectators().initFrom(ReportTestUtil.source(), json);
		assertArrayEquals(original.getSpectatorRollHome(), restored.getSpectatorRollHome());
		assertEquals(original.getSpectatorsHome(), restored.getSpectatorsHome());
		assertEquals(original.getFameHome(), restored.getFameHome());
		assertArrayEquals(original.getSpectatorRollAway(), restored.getSpectatorRollAway());
		assertEquals(original.getSpectatorsAway(), restored.getSpectatorsAway());
		assertEquals(original.getFameAway(), restored.getFameAway());
	}

	@Test
	public void toJsonValueHasReportId() {
		JsonObject json = make().toJsonValue();
		assertEquals("spectators", json.get("reportId").asString());
	}
}
