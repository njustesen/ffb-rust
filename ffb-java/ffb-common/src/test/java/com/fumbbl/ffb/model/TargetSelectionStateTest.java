package com.fumbbl.ffb.model;

import com.fumbbl.ffb.FactoryType;
import com.fumbbl.ffb.factory.SkillFactory;
import com.fumbbl.ffb.model.skill.Skill;
import com.fumbbl.ffb.net.NetCommandTestUtil;

import com.eclipsesource.json.JsonValue;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirrors the surviving Rust tests in crates/ffb-model/src/model/target_selection_state.rs for {@link TargetSelectionState}.
 */
public class TargetSelectionStateTest {

	@Test
	public void cancelSetsCanceled() {
		TargetSelectionState s = new TargetSelectionState();
		s.cancel();
		assertTrue(s.isCanceled());
	}

	@Test
	public void commitIsIdempotent() {
		TargetSelectionState s = new TargetSelectionState();
		s.commit(null);
		s.commit(null);
		assertTrue(s.isCommitted());
	}

	@Test
	public void serdeRoundTrip() {
		SkillFactory skillFactory = NetCommandTestUtil.gameSource().getFactory(FactoryType.Factory.SKILL);
		Skill dodge = skillFactory.forName("Dodge");
		TargetSelectionState s = new TargetSelectionState("p1");
		s.select();
		s.commit(null);
		s.addUsedSkill(dodge);
		JsonValue json = s.toJsonValue();
		TargetSelectionState back = new TargetSelectionState().initFrom(NetCommandTestUtil.gameSource(), json);
		assertTrue(back.isSelected());
		assertTrue(back.isCommitted());
		assertTrue(back.getUsedSkills().contains(dodge));
	}
}
