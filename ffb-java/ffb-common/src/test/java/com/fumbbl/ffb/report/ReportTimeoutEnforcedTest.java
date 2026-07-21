package com.fumbbl.ffb.report;

import com.eclipsesource.json.JsonObject;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

public class ReportTimeoutEnforcedTest {

	private ReportTimeoutEnforced make() {
		return new ReportTimeoutEnforced("Coach McCoach");
	}

	@Test
	public void serializationRoundTrip() {
		ReportTimeoutEnforced original = make();
		JsonObject json = original.toJsonValue();
		ReportTimeoutEnforced restored = new ReportTimeoutEnforced().initFrom(ReportTestUtil.source(), json);
		assertEquals(original.getCoach(), restored.getCoach());
	}

	@Test
	public void toJsonValueHasReportId() {
		JsonObject json = make().toJsonValue();
		assertEquals("timeoutEnforced", json.get("reportId").asString());
	}
}
