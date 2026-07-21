package com.fumbbl.ffb.report.mixed;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.report.ReportTestUtil;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

public class ReportLookIntoMyEyesRollTest {

	private ReportLookIntoMyEyesRoll make() {
		return new ReportLookIntoMyEyesRoll("p1", true, 4, false);
	}

	@Test
	public void serializationRoundTrip() {
		ReportLookIntoMyEyesRoll original = make();
		JsonObject json = (JsonObject) original.toJsonValue();
		ReportLookIntoMyEyesRoll restored = (ReportLookIntoMyEyesRoll) new ReportLookIntoMyEyesRoll().initFrom(ReportTestUtil.source(), json);
		assertEquals(original.getPlayerId(), restored.getPlayerId());
		assertEquals(original.isSuccessful(), restored.isSuccessful());
		assertEquals(original.getRoll(), restored.getRoll());
		assertEquals(original.getMinimumRoll(), restored.getMinimumRoll());
		assertEquals(original.isReRolled(), restored.isReRolled());
	}

	@Test
	public void toJsonValueHasReportId() {
		JsonObject json = (JsonObject) make().toJsonValue();
		assertEquals("lookIntoMyEyesRoll", json.get("reportId").asString());
	}
}
