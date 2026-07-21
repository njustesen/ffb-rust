package com.fumbbl.ffb.report.bb2025;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.report.ReportTestUtil;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

public class ReportSaboteurRollTest {
	private ReportSaboteurRoll make() {
		return new ReportSaboteurRoll("p1", true, 4, 3, false);
	}

	@Test
	public void serializationRoundTrip() {
		ReportSaboteurRoll original = make();
		JsonObject json = original.toJsonValue();
		ReportSaboteurRoll restored = (ReportSaboteurRoll) new ReportSaboteurRoll().initFrom(ReportTestUtil.source(), json);
		assertEquals(original.getPlayerId(), restored.getPlayerId());
		assertEquals(original.isSuccessful(), restored.isSuccessful());
		assertEquals(original.getRoll(), restored.getRoll());
		assertEquals(original.getMinimumRoll(), restored.getMinimumRoll());
		assertEquals(original.isReRolled(), restored.isReRolled());
	}

	@Test
	public void toJsonValueHasReportId() {
		JsonObject json = make().toJsonValue();
		assertEquals("saboteurRoll", json.get("reportId").asString());
	}
}
