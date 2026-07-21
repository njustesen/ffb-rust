package com.fumbbl.ffb.report.mixed;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.report.ReportTestUtil;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

public class ReportKickoffTimeoutTest {

	private ReportKickoffTimeout make() {
		return new ReportKickoffTimeout(4, 1);
	}

	@Test
	public void serializationRoundTrip() {
		ReportKickoffTimeout original = make();
		JsonObject json = (JsonObject) original.toJsonValue();
		ReportKickoffTimeout restored = new ReportKickoffTimeout().initFrom(ReportTestUtil.source(), json);
		assertEquals(original.getTurnModifier(), restored.getTurnModifier());
		assertEquals(original.getTurnNumber(), restored.getTurnNumber());
	}

	@Test
	public void toJsonValueHasReportId() {
		JsonObject json = (JsonObject) make().toJsonValue();
		assertEquals("kickoffTimeout", json.get("reportId").asString());
	}
}
