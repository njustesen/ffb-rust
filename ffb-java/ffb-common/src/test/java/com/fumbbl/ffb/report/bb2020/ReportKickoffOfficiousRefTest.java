package com.fumbbl.ffb.report.bb2020;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.report.ReportTestUtil;
import org.junit.jupiter.api.Test;

import java.util.Collections;

import static org.junit.jupiter.api.Assertions.assertEquals;

public class ReportKickoffOfficiousRefTest {
	private ReportKickoffOfficiousRef make() {
		return new ReportKickoffOfficiousRef(3, 2, Collections.singletonList("p1"));
	}

	@Test
	public void serializationRoundTrip() {
		ReportKickoffOfficiousRef original = make();
		JsonObject json = original.toJsonValue();
		ReportKickoffOfficiousRef restored = new ReportKickoffOfficiousRef().initFrom(ReportTestUtil.source(), json);
		assertEquals(original.getRollHome(), restored.getRollHome());
		assertEquals(original.getRollAway(), restored.getRollAway());
		assertEquals(original.getPlayerIds(), restored.getPlayerIds());
	}

	@Test
	public void toJsonValueHasReportId() {
		JsonObject json = make().toJsonValue();
		assertEquals("kickoffOfficiousRef", json.get("reportId").asString());
	}
}
