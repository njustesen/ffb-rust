package com.fumbbl.ffb.report.mixed;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.modifiers.RollModifier;
import com.fumbbl.ffb.report.ReportTestUtil;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertArrayEquals;
import static org.junit.jupiter.api.Assertions.assertEquals;

public class ReportPickupRollTest {

	private ReportPickupRoll make() {
		return new ReportPickupRoll("p1", true, 4, 3, false, new RollModifier<?>[0]);
	}

	@Test
	public void serializationRoundTrip() {
		ReportPickupRoll original = make();
		JsonObject json = original.toJsonValue().asObject();
		ReportPickupRoll restored = (ReportPickupRoll) new ReportPickupRoll().initFrom(ReportTestUtil.source(), json);
		assertEquals(original.getPlayerId(), restored.getPlayerId());
		assertEquals(original.isSuccessful(), restored.isSuccessful());
		assertEquals(original.getRoll(), restored.getRoll());
		assertEquals(original.getMinimumRoll(), restored.getMinimumRoll());
		assertEquals(original.isReRolled(), restored.isReRolled());
		assertArrayEquals(original.getRollModifiers(), restored.getRollModifiers());
	}

	@Test
	public void toJsonValueHasReportId() {
		JsonObject json = make().toJsonValue().asObject();
		assertEquals("pickUpRoll", json.get("reportId").asString());
	}
}
