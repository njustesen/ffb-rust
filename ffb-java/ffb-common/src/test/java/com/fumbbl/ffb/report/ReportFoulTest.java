package com.fumbbl.ffb.report;

import com.eclipsesource.json.JsonObject;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

public class ReportFoulTest {

	private ReportFoul make() {
		return new ReportFoul("defender1");
	}

	@Test
	public void serializationRoundTrip() {
		ReportFoul original = make();
		JsonObject json = original.toJsonValue();
		ReportFoul restored = new ReportFoul().initFrom(ReportTestUtil.source(), json);
		assertEquals(original.getDefenderId(), restored.getDefenderId());
	}

	@Test
	public void toJsonValueHasReportId() {
		JsonObject json = make().toJsonValue();
		assertEquals("foul", json.get("reportId").asString());
	}
}
