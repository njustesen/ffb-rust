package com.fumbbl.ffb.report.bb2025;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.report.ReportTestUtil;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

public class ReportChompRemovedTest {
	private ReportChompRemoved make() {
		return new ReportChompRemoved("p1", true);
	}

	@Test
	public void serializationRoundTrip() {
		ReportChompRemoved original = make();
		JsonObject json = original.toJsonValue();
		ReportChompRemoved restored = new ReportChompRemoved().initFrom(ReportTestUtil.source(), json);
		assertEquals(original.getPlayer(), restored.getPlayer());
		assertEquals(original.isSuccessful(), restored.isSuccessful());
	}

	@Test
	public void toJsonValueHasReportId() {
		JsonObject json = make().toJsonValue();
		assertEquals("chompRemoved", json.get("reportId").asString());
	}
}
