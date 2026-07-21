package com.fumbbl.ffb.report.bb2025;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.report.ReportTestUtil;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

public class ReportMascotUsedTest {
	private ReportMascotUsed make() {
		return new ReportMascotUsed("team1", 4, 5, true, false);
	}

	@Test
	public void serializationRoundTrip() {
		ReportMascotUsed original = make();
		JsonObject json = original.toJsonValue();
		ReportMascotUsed restored = new ReportMascotUsed().initFrom(ReportTestUtil.source(), json);
		assertEquals(original.getTeamId(), restored.getTeamId());
		assertEquals(original.getMinimumRoll(), restored.getMinimumRoll());
		assertEquals(original.getRoll(), restored.getRoll());
		assertEquals(original.isSuccessful(), restored.isSuccessful());
		assertEquals(original.isFallback(), restored.isFallback());
	}

	@Test
	public void toJsonValueHasReportId() {
		JsonObject json = make().toJsonValue();
		assertEquals("mascotUsed", json.get("reportId").asString());
	}
}
