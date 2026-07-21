package com.fumbbl.ffb.report.mixed;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.report.ReportTestUtil;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

public class ReportPumpUpTheCrowdReRollTest {

	private ReportPumpUpTheCrowdReRoll make() {
		return new ReportPumpUpTheCrowdReRoll("p1");
	}

	@Test
	public void serializationRoundTrip() {
		ReportPumpUpTheCrowdReRoll original = make();
		JsonObject json = original.toJsonValue().asObject();
		ReportPumpUpTheCrowdReRoll restored = new ReportPumpUpTheCrowdReRoll().initFrom(ReportTestUtil.source(), json);
		assertEquals(original.getPlayerId(), restored.getPlayerId());
	}

	@Test
	public void toJsonValueHasReportId() {
		JsonObject json = make().toJsonValue().asObject();
		assertEquals("pumpUpTheCrowdReRoll", json.get("reportId").asString());
	}
}
