package com.fumbbl.ffb.report.bb2025;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.report.ReportTestUtil;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

public class ReportPickupRollTest {
	private ReportPickupRoll make() {
		return new ReportPickupRoll("p1", true, 4, 3, false, null, true);
	}

	@Test
	public void serializationRoundTrip() {
		ReportPickupRoll original = make();
		JsonObject json = original.toJsonValue();
		ReportPickupRoll restored = (ReportPickupRoll) new ReportPickupRoll().initFrom(ReportTestUtil.source(), json);
		assertEquals(original.getPlayerId(), restored.getPlayerId());
		assertEquals(original.isSuccessful(), restored.isSuccessful());
		assertEquals(original.getRoll(), restored.getRoll());
		assertEquals(original.getMinimumRoll(), restored.getMinimumRoll());
		assertEquals(original.isReRolled(), restored.isReRolled());
		assertEquals(original.isSecureTheBall(), restored.isSecureTheBall());
	}

	@Test
	public void toJsonValueHasReportId() {
		JsonObject json = make().toJsonValue();
		assertEquals("pickUpRoll", json.get("reportId").asString());
	}
}
