package com.fumbbl.ffb.report;

import com.eclipsesource.json.JsonObject;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertArrayEquals;
import static org.junit.jupiter.api.Assertions.assertEquals;

public class ReportFoulAppearanceRollTest {

	private ReportFoulAppearanceRoll make() {
		return new ReportFoulAppearanceRoll("p1", false, 2, 3, true, null, "d1");
	}

	@Test
	public void serializationRoundTrip() {
		ReportFoulAppearanceRoll original = make();
		JsonObject json = original.toJsonValue();
		ReportFoulAppearanceRoll restored = new ReportFoulAppearanceRoll().initFrom(ReportTestUtil.source(), json);
		assertEquals(original.getPlayerId(), restored.getPlayerId());
		assertEquals(original.isSuccessful(), restored.isSuccessful());
		assertEquals(original.getRoll(), restored.getRoll());
		assertEquals(original.getMinimumRoll(), restored.getMinimumRoll());
		assertEquals(original.isReRolled(), restored.isReRolled());
		assertArrayEquals(original.getRollModifiers(), restored.getRollModifiers());
		assertEquals(original.getDefenderId(), restored.getDefenderId());
	}

	@Test
	public void toJsonValueHasReportId() {
		JsonObject json = make().toJsonValue();
		assertEquals("foulAppearanceRoll", json.get("reportId").asString());
	}
}
