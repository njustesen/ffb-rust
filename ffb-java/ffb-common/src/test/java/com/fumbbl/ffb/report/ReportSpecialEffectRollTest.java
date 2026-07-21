package com.fumbbl.ffb.report;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.SpecialEffect;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

public class ReportSpecialEffectRollTest {

	private ReportSpecialEffectRoll make() {
		return new ReportSpecialEffectRoll(SpecialEffect.LIGHTNING, "p1", 4, true);
	}

	@Test
	public void serializationRoundTrip() {
		ReportSpecialEffectRoll original = make();
		JsonObject json = original.toJsonValue();
		ReportSpecialEffectRoll restored = new ReportSpecialEffectRoll().initFrom(ReportTestUtil.source(), json);
		assertEquals(original.getSpecialEffect(), restored.getSpecialEffect());
		assertEquals(original.getPlayerId(), restored.getPlayerId());
		assertEquals(original.getRoll(), restored.getRoll());
		assertEquals(original.isSuccessful(), restored.isSuccessful());
	}

	@Test
	public void toJsonValueHasReportId() {
		JsonObject json = make().toJsonValue();
		assertEquals("spellEffectRoll", json.get("reportId").asString());
	}
}
