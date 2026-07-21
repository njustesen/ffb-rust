package com.fumbbl.ffb.report.mixed;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.report.ReportTestUtil;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertArrayEquals;
import static org.junit.jupiter.api.Assertions.assertEquals;

public class ReportCatchOfTheDayRollTest {

	private ReportCatchOfTheDayRoll make() {
		return new ReportCatchOfTheDayRoll("p1", true, 4, 2, false);
	}

	@Test
	public void serializationRoundTrip() {
		ReportCatchOfTheDayRoll original = make();
		JsonObject json = (JsonObject) original.toJsonValue();
		ReportCatchOfTheDayRoll restored = (ReportCatchOfTheDayRoll) new ReportCatchOfTheDayRoll().initFrom(ReportTestUtil.source(), json);
		assertEquals(original.getPlayerId(), restored.getPlayerId());
		assertEquals(original.isSuccessful(), restored.isSuccessful());
		assertEquals(original.getRoll(), restored.getRoll());
		assertEquals(original.getMinimumRoll(), restored.getMinimumRoll());
		assertEquals(original.isReRolled(), restored.isReRolled());
		assertArrayEquals(original.getRollModifiers(), restored.getRollModifiers());
	}

	@Test
	public void toJsonValueHasReportId() {
		JsonObject json = (JsonObject) make().toJsonValue();
		assertEquals("catchOfTheDay", json.get("reportId").asString());
	}
}
