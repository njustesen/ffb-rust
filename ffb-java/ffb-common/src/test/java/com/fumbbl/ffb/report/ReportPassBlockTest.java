package com.fumbbl.ffb.report;

import com.eclipsesource.json.JsonObject;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

public class ReportPassBlockTest {

	private ReportPassBlock make() {
		return new ReportPassBlock("team1", true);
	}

	@Test
	public void serializationRoundTrip() {
		ReportPassBlock original = make();
		JsonObject json = original.toJsonValue();
		ReportPassBlock restored = new ReportPassBlock().initFrom(ReportTestUtil.source(), json);
		assertEquals(original.getTeamId(), restored.getTeamId());
		assertEquals(original.isPassBlockAvailable(), restored.isPassBlockAvailable());
	}

	@Test
	public void toJsonValueHasReportId() {
		JsonObject json = make().toJsonValue();
		assertEquals("passBlock", json.get("reportId").asString());
	}
}
