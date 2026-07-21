package com.fumbbl.ffb.report.mixed;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.FactoryType;
import com.fumbbl.ffb.factory.SkillFactory;
import com.fumbbl.ffb.model.skill.Skill;
import com.fumbbl.ffb.report.ReportTestUtil;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNotNull;

public class ReportSkillWastedTest {

	private static Skill skill(String name) {
		SkillFactory factory = ReportTestUtil.source().getFactory(FactoryType.Factory.SKILL);
		return factory.forName(name);
	}

	private ReportSkillWasted make() {
		return new ReportSkillWasted("p1", null);
	}

	@Test
	public void serializationRoundTrip() {
		Skill dodge = skill("Dodge");
		assertNotNull(dodge);
		ReportSkillWasted original = new ReportSkillWasted("p2", dodge);
		JsonObject json = original.toJsonValue().asObject();
		ReportSkillWasted restored = new ReportSkillWasted().initFrom(ReportTestUtil.source(), json);
		assertEquals(original.getPlayerId(), restored.getPlayerId());
		assertNotNull(restored.getSkill());
		assertEquals(original.getSkill().getName(), restored.getSkill().getName());
	}

	@Test
	public void toJsonValueHasReportId() {
		JsonObject json = make().toJsonValue().asObject();
		assertEquals("skillWasted", json.get("reportId").asString());
	}
}
