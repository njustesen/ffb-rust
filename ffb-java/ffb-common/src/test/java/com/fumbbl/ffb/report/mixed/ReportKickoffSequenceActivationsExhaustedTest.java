package com.fumbbl.ffb.report.mixed;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.report.ReportTestUtil;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

public class ReportKickoffSequenceActivationsExhaustedTest {

	private ReportKickoffSequenceActivationsExhausted make() {
		return new ReportKickoffSequenceActivationsExhausted(true);
	}

	@Test
	public void serializationRoundTrip() {
		ReportKickoffSequenceActivationsExhausted original = make();
		JsonObject json = (JsonObject) original.toJsonValue();
		ReportKickoffSequenceActivationsExhausted restored = (ReportKickoffSequenceActivationsExhausted) new ReportKickoffSequenceActivationsExhausted().initFrom(ReportTestUtil.source(), json);
		assertEquals(original.isLimitReached(), restored.isLimitReached());
	}

	@Test
	public void toJsonValueHasReportId() {
		JsonObject json = (JsonObject) make().toJsonValue();
		assertEquals("kickoffSequenceActivationsExhausted", json.get("reportId").asString());
	}
}
