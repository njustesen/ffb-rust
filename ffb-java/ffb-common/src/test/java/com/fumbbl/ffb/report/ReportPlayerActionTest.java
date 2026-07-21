package com.fumbbl.ffb.report;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.PlayerAction;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

public class ReportPlayerActionTest {

	private ReportPlayerAction make() {
		return new ReportPlayerAction("p1", PlayerAction.MOVE);
	}

	@Test
	public void serializationRoundTrip() {
		ReportPlayerAction original = make();
		JsonObject json = original.toJsonValue();
		ReportPlayerAction restored = new ReportPlayerAction().initFrom(ReportTestUtil.source(), json);
		assertEquals(original.getActingPlayerId(), restored.getActingPlayerId());
		assertEquals(original.getPlayerAction(), restored.getPlayerAction());
	}

	@Test
	public void toJsonValueHasReportId() {
		JsonObject json = make().toJsonValue();
		assertEquals("playerAction", json.get("reportId").asString());
	}
}
