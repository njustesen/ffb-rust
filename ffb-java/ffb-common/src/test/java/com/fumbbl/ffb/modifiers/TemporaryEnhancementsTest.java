package com.fumbbl.ffb.modifiers;

import com.fumbbl.ffb.FactoryType;
import com.fumbbl.ffb.factory.SkillFactory;
import com.fumbbl.ffb.mechanics.StatsMechanic;
import com.fumbbl.ffb.model.property.ISkillProperty;
import com.fumbbl.ffb.model.property.NamedProperties;
import com.fumbbl.ffb.model.skill.Skill;
import com.fumbbl.ffb.model.skill.SkillClassWithValue;
import com.fumbbl.ffb.net.NetCommandTestUtil;
import org.junit.jupiter.api.Test;

import java.util.HashSet;
import java.util.Set;

import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirror of ffb-rust crates/ffb-mechanics/src/modifiers/temporary_enhancements.rs tests.
 * The Rust struct is an acknowledged placeholder (String sets for skills/properties); Java uses the
 * real typed collections — Set&lt;TemporaryStatModifier&gt; / Set&lt;SkillClassWithValue&gt; /
 * Set&lt;ISkillProperty&gt; via the fluent withX builders. The Rust String inserts map by intent to
 * these typed adds; the Rust-only 4th field (stat_modifiers) is a placeholder artifact.
 */
public class TemporaryEnhancementsTest {

	private final StatsMechanic mechanic = new com.fumbbl.ffb.mechanics.mixed.StatsMechanic();

	private Skill skill(String name) {
		SkillFactory factory = NetCommandTestUtil.gameSource().getFactory(FactoryType.Factory.SKILL);
		return factory.forName(name);
	}

	// rust: new_starts_empty
	@Test
	public void newStartsEmpty() {
		TemporaryEnhancements e = new TemporaryEnhancements();
		assertTrue(e.getModifiers().isEmpty());
		assertTrue(e.getSkills().isEmpty());
		assertTrue(e.getProperties().isEmpty());
	}

	// rust: can_add_modifier_string
	@Test
	public void canAddModifier() {
		Set<TemporaryStatModifier> mods = new HashSet<>();
		mods.add(new TemporaryStatIncrementer(PlayerStatKey.ST, mechanic));
		TemporaryEnhancements e = new TemporaryEnhancements().withModifiers(mods);
		assertFalse(e.getModifiers().isEmpty());
	}

	// rust: can_add_skill_and_property
	@Test
	public void canAddSkillAndProperty() {
		Set<SkillClassWithValue> skills = new HashSet<>();
		skills.add(new SkillClassWithValue(skill("Dodge").getClass()));
		Set<ISkillProperty> properties = new HashSet<>();
		properties.add(NamedProperties.addStrengthOnBlitz);
		TemporaryEnhancements e = new TemporaryEnhancements().withSkills(skills).withProperties(properties);
		assertFalse(e.getSkills().isEmpty());
		assertTrue(e.getProperties().contains(NamedProperties.addStrengthOnBlitz));
	}

	// rust: default_is_same_as_new
	@Test
	public void defaultIsSameAsNew() {
		TemporaryEnhancements e = new TemporaryEnhancements();
		assertTrue(e.getModifiers().isEmpty());
		assertTrue(e.getSkills().isEmpty());
	}

	// rust: insert_skill_string_then_not_empty
	@Test
	public void insertSkillThenNotEmpty() {
		Set<SkillClassWithValue> skills = new HashSet<>();
		skills.add(new SkillClassWithValue(skill("Dodge").getClass()));
		TemporaryEnhancements e = new TemporaryEnhancements().withSkills(skills);
		assertFalse(e.getSkills().isEmpty());
	}
}
