package com.fumbbl.ffb.report;

import com.eclipsesource.json.JsonObject;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

public class ReportPilingOnTest {

	private ReportPilingOn make() {
		return new ReportPilingOn("p1", true, false);
	}

	@Test
	public void serializationRoundTrip() {
		ReportPilingOn original = make();
		JsonObject json = original.toJsonValue();
		ReportPilingOn restored = new ReportPilingOn().initFrom(ReportTestUtil.source(), json);
		assertEquals(original.getPlayerId(), restored.getPlayerId());
		assertEquals(original.isUsed(), restored.isUsed());
		assertEquals(original.isReRollInjury(), restored.isReRollInjury());
	}

	@Test
	public void toJsonValueHasReportId() {
		JsonObject json = make().toJsonValue();
		assertEquals("pilingOn", json.get("reportId").asString());
	}
}
