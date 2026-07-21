package com.fumbbl.ffb.report.bb2025;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.report.ReportTestUtil;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

public class ReportDodgySnackRollTest {
	private ReportDodgySnackRoll make() {
		return new ReportDodgySnackRoll(4, "p1");
	}

	@Test
	public void serializationRoundTrip() {
		ReportDodgySnackRoll original = make();
		JsonObject json = original.toJsonValue();
		ReportDodgySnackRoll restored = new ReportDodgySnackRoll().initFrom(ReportTestUtil.source(), json);
		assertEquals(original.getRoll(), restored.getRoll());
		assertEquals(original.getPlayerId(), restored.getPlayerId());
	}

	@Test
	public void toJsonValueHasReportId() {
		JsonObject json = make().toJsonValue();
		assertEquals("dodgySnackRoll", json.get("reportId").asString());
	}
}
