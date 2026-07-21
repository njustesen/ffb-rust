package com.fumbbl.ffb.report.bb2025;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.model.Keyword;
import com.fumbbl.ffb.report.ReportTestUtil;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

public class ReportGettingEvenRollTest {
	private ReportGettingEvenRoll make() {
		return new ReportGettingEvenRoll("p1", true, 4, 3, false, Keyword.ELF);
	}

	@Test
	public void serializationRoundTrip() {
		ReportGettingEvenRoll original = make();
		JsonObject json = original.toJsonValue();
		ReportGettingEvenRoll restored = (ReportGettingEvenRoll) new ReportGettingEvenRoll().initFrom(ReportTestUtil.source(), json);
		assertEquals(original.getPlayerId(), restored.getPlayerId());
		assertEquals(original.isSuccessful(), restored.isSuccessful());
		assertEquals(original.getRoll(), restored.getRoll());
		assertEquals(original.getMinimumRoll(), restored.getMinimumRoll());
		assertEquals(original.isReRolled(), restored.isReRolled());
		assertEquals(original.getKeyword(), restored.getKeyword());
	}

	@Test
	public void toJsonValueHasReportId() {
		JsonObject json = make().toJsonValue();
		assertEquals("gettingEvenRoll", json.get("reportId").asString());
	}
}
