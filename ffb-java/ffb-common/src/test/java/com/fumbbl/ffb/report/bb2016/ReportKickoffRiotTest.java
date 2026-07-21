package com.fumbbl.ffb.report.bb2016;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.report.ReportTestUtil;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

public class ReportKickoffRiotTest {

	private ReportKickoffRiot make() {
		return new ReportKickoffRiot(3, -1);
	}

	@Test
	public void serializationRoundTrip() {
		ReportKickoffRiot original = make();
		JsonObject json = original.toJsonValue();
		ReportKickoffRiot restored = new ReportKickoffRiot().initFrom(ReportTestUtil.source(), json);
		assertEquals(original.getRoll(), restored.getRoll());
		assertEquals(original.getTurnModifier(), restored.getTurnModifier());
	}

	@Test
	public void toJsonValueHasReportId() {
		JsonObject json = make().toJsonValue();
		assertEquals("kickoffRiot", json.get("reportId").asString());
	}
}
