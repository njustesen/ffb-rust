package com.fumbbl.ffb.report;

import com.eclipsesource.json.JsonObject;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

public class ReportBiteSpectatorTest {

	private ReportBiteSpectator make() {
		return new ReportBiteSpectator("p1");
	}

	@Test
	public void serializationRoundTrip() {
		ReportBiteSpectator original = make();
		JsonObject json = original.toJsonValue();
		ReportBiteSpectator restored = new ReportBiteSpectator().initFrom(ReportTestUtil.source(), json);
		assertEquals(original.getPlayerId(), restored.getPlayerId());
	}

	@Test
	public void toJsonValueHasReportId() {
		JsonObject json = make().toJsonValue();
		assertEquals("biteSpectator", json.get("reportId").asString());
	}
}
