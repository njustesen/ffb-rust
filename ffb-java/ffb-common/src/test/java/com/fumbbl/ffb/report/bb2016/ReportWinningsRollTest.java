package com.fumbbl.ffb.report.bb2016;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.report.ReportTestUtil;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

public class ReportWinningsRollTest {

	private ReportWinningsRoll make() {
		return new ReportWinningsRoll(4, 40000, 2, 20000);
	}

	@Test
	public void serializationRoundTrip() {
		ReportWinningsRoll original = make();
		JsonObject json = original.toJsonValue();
		ReportWinningsRoll restored = new ReportWinningsRoll().initFrom(ReportTestUtil.source(), json);
		assertEquals(original.getWinningsRollHome(), restored.getWinningsRollHome());
		assertEquals(original.getWinningsHome(), restored.getWinningsHome());
		assertEquals(original.getWinningsRollAway(), restored.getWinningsRollAway());
		assertEquals(original.getWinningsAway(), restored.getWinningsAway());
	}

	@Test
	public void toJsonValueHasReportId() {
		JsonObject json = make().toJsonValue();
		assertEquals("winningsRoll", json.get("reportId").asString());
	}
}
