package com.fumbbl.ffb.report;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.FactoryType.Factory;
import com.fumbbl.ffb.SkillUse;
import com.fumbbl.ffb.factory.SkillFactory;
import com.fumbbl.ffb.model.skill.Skill;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

public class ReportSkillUseTest {

	private ReportSkillUse make() {
		SkillFactory skillFactory = ReportTestUtil.source().getFactory(Factory.SKILL);
		Skill skill = skillFactory.forName("Block");
		return new ReportSkillUse("p1", skill, true, SkillUse.BRING_DOWN_OPPONENT);
	}

	@Test
	public void serializationRoundTrip() {
		ReportSkillUse original = make();
		JsonObject json = original.toJsonValue();
		ReportSkillUse restored = new ReportSkillUse().initFrom(ReportTestUtil.source(), json);
		assertEquals(original.getPlayerId(), restored.getPlayerId());
		assertEquals(original.getSkill(), restored.getSkill());
		assertEquals(original.isUsed(), restored.isUsed());
		assertEquals(original.getSkillUse(), restored.getSkillUse());
	}

	@Test
	public void toJsonValueHasReportId() {
		JsonObject json = make().toJsonValue();
		assertEquals("skillUse", json.get("reportId").asString());
	}
}
