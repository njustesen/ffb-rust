package com.fumbbl.ffb.dialog;

import com.fumbbl.ffb.FactoryType;
import com.fumbbl.ffb.factory.SkillFactory;
import com.fumbbl.ffb.model.skill.Skill;
import com.fumbbl.ffb.net.NetCommandTestUtil;

import com.eclipsesource.json.JsonValue;
import org.junit.jupiter.api.Test;

import java.util.Arrays;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNotNull;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirrors the surviving Rust tests in
 * crates/ffb-model/src/dialog/dialog_re_roll_parameter.rs for
 * {@link DialogReRollParameter}.
 */
public class DialogReRollParameterTest {

	private static Skill skill(String name) {
		SkillFactory factory = NetCommandTestUtil.gameSource().getFactory(FactoryType.Factory.SKILL);
		return factory.forName(name);
	}

	@Test
	public void serdeRoundTrip() {
		Skill pro = skill("Pro");
		DialogReRollParameter p = new DialogReRollParameter("p42", null, 4, true, false, false, pro, null, null, null,
			null, Arrays.asList("Roll for it!"));
		JsonValue json = p.toJsonValue();
		DialogReRollParameter back = (DialogReRollParameter) new DialogReRollParameter()
			.initFrom(NetCommandTestUtil.gameSource(), json);
		assertEquals("p42", back.getPlayerId());
		assertEquals(4, back.getMinimumRoll());
		assertTrue(back.isTeamReRollOption());
		assertNotNull(back.getReRollSkill());
		assertEquals("Pro", back.getReRollSkill().getName());
	}

}
