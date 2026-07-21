package com.fumbbl.ffb.report.bb2025;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.report.ReportTestUtil;
import org.junit.jupiter.api.Test;

import java.util.Collections;

import static org.junit.jupiter.api.Assertions.assertEquals;

public class ReportKickoffDodgySnackTest {
	private ReportKickoffDodgySnack make() {
		return new ReportKickoffDodgySnack(3, 4, Collections.singletonList("p1"));
	}

	@Test
	public void serializationRoundTrip() {
		ReportKickoffDodgySnack original = make();
		JsonObject json = original.toJsonValue();
		ReportKickoffDodgySnack restored = new ReportKickoffDodgySnack().initFrom(ReportTestUtil.source(), json);
		assertEquals(original.getRollHome(), restored.getRollHome());
		assertEquals(original.getRollAway(), restored.getRollAway());
		assertEquals(original.getPlayerIds(), restored.getPlayerIds());
	}

	@Test
	public void toJsonValueHasReportId() {
		JsonObject json = make().toJsonValue();
		assertEquals("kickoffDodgySnack", json.get("reportId").asString());
	}
}
