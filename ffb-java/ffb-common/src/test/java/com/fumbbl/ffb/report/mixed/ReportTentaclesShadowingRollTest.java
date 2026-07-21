package com.fumbbl.ffb.report.mixed;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.model.skill.Skill;
import com.fumbbl.ffb.report.ReportTestUtil;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

public class ReportTentaclesShadowingRollTest {

	private ReportTentaclesShadowingRoll make() {
		return new ReportTentaclesShadowingRoll((Skill) null, "d1", 4, true, 3, false);
	}

	@Test
	public void serializationRoundTrip() {
		ReportTentaclesShadowingRoll original = make();
		JsonObject json = original.toJsonValue().asObject();
		ReportTentaclesShadowingRoll restored = new ReportTentaclesShadowingRoll().initFrom(ReportTestUtil.source(), json);
		assertEquals(original.getSkill(), restored.getSkill());
		assertEquals(original.getDefenderId(), restored.getDefenderId());
		assertEquals(original.getRoll(), restored.getRoll());
		assertEquals(original.isSuccessful(), restored.isSuccessful());
		assertEquals(original.getMinimumRoll(), restored.getMinimumRoll());
		assertEquals(original.isReRolled(), restored.isReRolled());
	}

	@Test
	public void toJsonValueHasReportId() {
		JsonObject json = make().toJsonValue().asObject();
		assertEquals("tentaclesShadowingRoll", json.get("reportId").asString());
	}
}
