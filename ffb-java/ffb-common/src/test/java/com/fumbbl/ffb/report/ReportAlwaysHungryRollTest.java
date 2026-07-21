package com.fumbbl.ffb.report;

import com.eclipsesource.json.JsonObject;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertArrayEquals;
import static org.junit.jupiter.api.Assertions.assertEquals;

public class ReportAlwaysHungryRollTest {

	private ReportAlwaysHungryRoll make() {
		return new ReportAlwaysHungryRoll("p1", true, 4, 2, false, null);
	}

	@Test
	public void serializationRoundTrip() {
		ReportAlwaysHungryRoll original = make();
		JsonObject json = original.toJsonValue();
		ReportAlwaysHungryRoll restored = (ReportAlwaysHungryRoll) new ReportAlwaysHungryRoll().initFrom(ReportTestUtil.source(), json);
		assertEquals(original.getPlayerId(), restored.getPlayerId());
		assertEquals(original.isSuccessful(), restored.isSuccessful());
		assertEquals(original.getRoll(), restored.getRoll());
		assertEquals(original.getMinimumRoll(), restored.getMinimumRoll());
		assertEquals(original.isReRolled(), restored.isReRolled());
		assertArrayEquals(original.getRollModifiers(), restored.getRollModifiers());
	}

	@Test
	public void toJsonValueHasReportId() {
		JsonObject json = make().toJsonValue();
		assertEquals("alwaysHungryRoll", json.get("reportId").asString());
	}
}
