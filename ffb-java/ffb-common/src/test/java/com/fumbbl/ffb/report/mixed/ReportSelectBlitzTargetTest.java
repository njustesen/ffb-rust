package com.fumbbl.ffb.report.mixed;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.report.ReportTestUtil;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

public class ReportSelectBlitzTargetTest {

	private ReportSelectBlitzTarget make() {
		return new ReportSelectBlitzTarget("a1", "d1");
	}

	@Test
	public void serializationRoundTrip() {
		ReportSelectBlitzTarget original = make();
		JsonObject json = original.toJsonValue().asObject();
		ReportSelectBlitzTarget restored = new ReportSelectBlitzTarget().initFrom(ReportTestUtil.source(), json);
		assertEquals(original.getAttacker(), restored.getAttacker());
		assertEquals(original.getDefender(), restored.getDefender());
	}

	@Test
	public void toJsonValueHasReportId() {
		JsonObject json = make().toJsonValue().asObject();
		assertEquals("selectBlitzTarget", json.get("reportId").asString());
	}
}
