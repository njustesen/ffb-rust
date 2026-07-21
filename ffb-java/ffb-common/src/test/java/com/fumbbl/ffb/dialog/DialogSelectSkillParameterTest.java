package com.fumbbl.ffb.dialog;

import com.fumbbl.ffb.FactoryType;
import com.fumbbl.ffb.SkillChoiceMode;
import com.fumbbl.ffb.factory.SkillFactory;
import com.fumbbl.ffb.model.skill.Skill;
import com.fumbbl.ffb.net.NetCommandTestUtil;

import com.eclipsesource.json.JsonValue;
import org.junit.jupiter.api.Test;

import java.util.Arrays;

import static org.junit.jupiter.api.Assertions.assertEquals;

/**
 * Mirrors the surviving Rust tests in
 * crates/ffb-model/src/dialog/dialog_select_skill_parameter.rs for
 * {@link DialogSelectSkillParameter}.
 */
public class DialogSelectSkillParameterTest {

	private static Skill skill(String name) {
		SkillFactory factory = NetCommandTestUtil.gameSource().getFactory(FactoryType.Factory.SKILL);
		return factory.forName(name);
	}

	@Test
	public void addSkillAndSerde() {
		Skill dodge = skill("Dodge");
		Skill block = skill("Block");
		DialogSelectSkillParameter p = new DialogSelectSkillParameter("player1", Arrays.asList(dodge, block),
			SkillChoiceMode.INTENSIVE_TRAINING);
		JsonValue json = p.toJsonValue();
		DialogSelectSkillParameter back = (DialogSelectSkillParameter) new DialogSelectSkillParameter()
			.initFrom(NetCommandTestUtil.gameSource(), json);
		assertEquals(2, back.getSkills().size());
		assertEquals("Dodge", back.getSkills().get(0).getName());
		assertEquals("Block", back.getSkills().get(1).getName());
		assertEquals("player1", back.getPlayerId());
	}

}
