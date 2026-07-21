package com.fumbbl.ffb.report.bb2016;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.FactoryType;
import com.fumbbl.ffb.factory.SkillFactory;
import com.fumbbl.ffb.model.skill.Skill;
import com.fumbbl.ffb.report.ReportTestUtil;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertArrayEquals;
import static org.junit.jupiter.api.Assertions.assertEquals;

public class ReportTentaclesShadowingRollTest {

	private Skill skill(String name) {
		SkillFactory factory = ReportTestUtil.source().getFactory(FactoryType.Factory.SKILL);
		return factory.forName(name);
	}

	private ReportTentaclesShadowingRoll make() {
		return new ReportTentaclesShadowingRoll(skill("Tentacles"), "d1", new int[]{3, 4}, false, 5, false);
	}

	@Test
	public void serializationRoundTrip() {
		ReportTentaclesShadowingRoll original = make();
		JsonObject json = original.toJsonValue();
		ReportTentaclesShadowingRoll restored = new ReportTentaclesShadowingRoll().initFrom(ReportTestUtil.source(), json);
		assertEquals(original.getSkill(), restored.getSkill());
		assertEquals(original.getDefenderId(), restored.getDefenderId());
		assertArrayEquals(original.getRoll(), restored.getRoll());
		assertEquals(original.isSuccessful(), restored.isSuccessful());
		assertEquals(original.getMinimumRoll(), restored.getMinimumRoll());
	}

	@Test
	public void toJsonValueHasReportId() {
		JsonObject json = make().toJsonValue();
		assertEquals("tentaclesShadowingRoll", json.get("reportId").asString());
	}
}
