package com.fumbbl.ffb.model.skill;

import com.fumbbl.ffb.ReRollSource;
import com.fumbbl.ffb.ReRolledAction;
import com.fumbbl.ffb.SkillCategory;
import com.fumbbl.ffb.factory.ReRollSourceFactory;
import com.fumbbl.ffb.factory.ReRolledActionFactory;

import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNotEquals;
import static org.junit.jupiter.api.Assertions.assertNotNull;
import static org.junit.jupiter.api.Assertions.assertNull;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirrors the surviving Rust tests in crates/ffb-model/src/model/skill/skill.rs for {@link Skill}.
 */
public class SkillTest {

	private static final class NamedSkill extends Skill {
		NamedSkill(String name, SkillCategory category) {
			super(name, category);
		}
	}

	@Test
	public void equalityBasedOnNameOnly() {
		Skill a = new NamedSkill("Block", SkillCategory.GENERAL);
		Skill b = new NamedSkill("Block", SkillCategory.STRENGTH);
		assertEquals(a, b);
		Skill c = new NamedSkill("Dodge", SkillCategory.GENERAL);
		assertNotEquals(a, c);
	}

	@Test
	public void orderingByName() {
		Skill a = new NamedSkill("Block", SkillCategory.GENERAL);
		Skill b = new NamedSkill("Dodge", SkillCategory.AGILITY);
		assertTrue(a.compareTo(b) < 0);
		assertTrue(b.compareTo(a) > 0);
	}

	@Test
	public void registerAndQueryRerollSource() {
		Skill s = new NamedSkill("Dodge", SkillCategory.AGILITY);
		ReRolledAction action = new ReRolledActionFactory().forName("DODGE");
		ReRollSource source = new ReRollSourceFactory().forName("Dodge");
		s.registerRerollSource(action, source);
		assertNotNull(s.getRerollSource(action));
		ReRolledAction missing = new ReRolledActionFactory().forName("BLOCK");
		assertNull(s.getRerollSource(missing));
	}
}
