package com.fumbbl.ffb.report;

import com.eclipsesource.json.JsonObject;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

public class ReportBlockTest {

	private ReportBlock make() {
		return new ReportBlock("def1");
	}

	@Test
	public void serializationRoundTrip() {
		ReportBlock original = make();
		JsonObject json = original.toJsonValue();
		ReportBlock restored = new ReportBlock().initFrom(ReportTestUtil.source(), json);
		assertEquals(original.getDefenderId(), restored.getDefenderId());
	}

	@Test
	public void toJsonValueHasReportId() {
		JsonObject json = make().toJsonValue();
		assertEquals("block", json.get("reportId").asString());
	}
}
