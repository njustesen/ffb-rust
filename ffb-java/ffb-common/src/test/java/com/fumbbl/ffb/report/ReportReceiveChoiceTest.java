package com.fumbbl.ffb.report;

import com.eclipsesource.json.JsonObject;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

public class ReportReceiveChoiceTest {

	private ReportReceiveChoice make() {
		return new ReportReceiveChoice("team1", true);
	}

	@Test
	public void serializationRoundTrip() {
		ReportReceiveChoice original = make();
		JsonObject json = original.toJsonValue();
		ReportReceiveChoice restored = new ReportReceiveChoice().initFrom(ReportTestUtil.source(), json);
		assertEquals(original.getTeamId(), restored.getTeamId());
		assertEquals(original.isReceiveChoice(), restored.isReceiveChoice());
	}

	@Test
	public void toJsonValueHasReportId() {
		JsonObject json = make().toJsonValue();
		assertEquals("receiveChoice", json.get("reportId").asString());
	}
}
