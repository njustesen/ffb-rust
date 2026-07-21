package com.fumbbl.ffb.report.bb2016;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.modifiers.RollModifier;
import com.fumbbl.ffb.report.ReportTestUtil;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

public class ReportDodgeRollTest {

	private ReportDodgeRoll make() {
		return new ReportDodgeRoll("p1", true, 4, 3, false, new RollModifier<?>[0]);
	}

	@Test
	public void serializationRoundTrip() {
		ReportDodgeRoll original = make();
		JsonObject json = original.toJsonValue();
		ReportDodgeRoll restored = (ReportDodgeRoll) new ReportDodgeRoll().initFrom(ReportTestUtil.source(), json);
		assertEquals(original.getPlayerId(), restored.getPlayerId());
		assertEquals(original.isSuccessful(), restored.isSuccessful());
		assertEquals(original.getRoll(), restored.getRoll());
	}

	@Test
	public void toJsonValueHasReportId() {
		JsonObject json = make().toJsonValue();
		assertEquals("dodgeRoll", json.get("reportId").asString());
	}
}
