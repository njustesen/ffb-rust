package com.fumbbl.ffb.report;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.PushbackMode;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

public class ReportPushbackTest {

	private ReportPushback make() {
		return new ReportPushback("def1", PushbackMode.REGULAR);
	}

	@Test
	public void serializationRoundTrip() {
		ReportPushback original = make();
		JsonObject json = original.toJsonValue();
		ReportPushback restored = new ReportPushback().initFrom(ReportTestUtil.source(), json);
		assertEquals(original.getDefenderId(), restored.getDefenderId());
		assertEquals(original.getPushbackMode(), restored.getPushbackMode());
	}

	@Test
	public void toJsonValueHasReportId() {
		JsonObject json = make().toJsonValue();
		assertEquals("pushback", json.get("reportId").asString());
	}
}
