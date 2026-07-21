package com.fumbbl.ffb.report;

import com.eclipsesource.json.JsonObject;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

public class ReportGameOptionsTest {

	private ReportGameOptions make() {
		return new ReportGameOptions();
	}

	@Test
	public void serializationRoundTrip() {
		ReportGameOptions original = make();
		JsonObject json = original.toJsonValue();
		ReportGameOptions restored = new ReportGameOptions().initFrom(ReportTestUtil.source(), json);
		// Java only serializes reportId; fields are not persisted
		assertEquals(original.getId(), restored.getId());
		assertEquals("gameOptions", json.get("reportId").asString());
	}

	@Test
	public void toJsonValueHasReportId() {
		JsonObject json = make().toJsonValue();
		assertEquals("gameOptions", json.get("reportId").asString());
	}
}
