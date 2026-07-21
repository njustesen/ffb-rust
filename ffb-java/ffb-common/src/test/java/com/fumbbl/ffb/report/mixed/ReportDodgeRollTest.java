package com.fumbbl.ffb.report.mixed;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.modifiers.RollModifier;
import com.fumbbl.ffb.modifiers.StatBasedRollModifier;
import com.fumbbl.ffb.report.ReportTestUtil;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertArrayEquals;
import static org.junit.jupiter.api.Assertions.assertEquals;

public class ReportDodgeRollTest {

	private ReportDodgeRoll make() {
		return new ReportDodgeRoll("p1", true, 4, 2, false, new RollModifier[0], new StatBasedRollModifier("mod1", 0));
	}

	@Test
	public void serializationRoundTrip() {
		ReportDodgeRoll original = make();
		JsonObject json = (JsonObject) original.toJsonValue();
		ReportDodgeRoll restored = (ReportDodgeRoll) new ReportDodgeRoll().initFrom(ReportTestUtil.source(), json);
		assertEquals(original.getPlayerId(), restored.getPlayerId());
		assertEquals(original.isSuccessful(), restored.isSuccessful());
		assertEquals(original.getRoll(), restored.getRoll());
		assertEquals(original.getMinimumRoll(), restored.getMinimumRoll());
		assertEquals(original.isReRolled(), restored.isReRolled());
		assertArrayEquals(original.getRollModifiers(), restored.getRollModifiers());
		assertEquals(original.getStatBasedRollModifier().getName(), restored.getStatBasedRollModifier().getName());
	}

	@Test
	public void toJsonValueHasReportId() {
		JsonObject json = (JsonObject) make().toJsonValue();
		assertEquals("dodgeRoll", json.get("reportId").asString());
	}
}
