package com.fumbbl.ffb.modifiers;

import com.fumbbl.ffb.FactoryType;
import com.fumbbl.ffb.factory.SkillFactory;
import com.fumbbl.ffb.model.skill.Skill;
import com.fumbbl.ffb.net.NetCommandTestUtil;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertNull;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirror of ffb-rust crates/ffb-mechanics/src/modifiers/static_armour_modifier.rs tests.
 * Java setRegisteredTo takes a Skill (Rust uses a skill-id string); a real Dodge Skill is
 * resolved from the game-source SkillFactory to exercise the registration getters.
 */
public class StaticArmourModifierTest {

	private Skill skill(String name) {
		SkillFactory factory = NetCommandTestUtil.gameSource().getFactory(FactoryType.Factory.SKILL);
		return factory.forName(name);
	}

	// rust: new_stores_name_modifier_and_foul_flag
	@Test
	public void newStoresNameModifierAndFoulFlag() {
		StaticArmourModifier m = new StaticArmourModifier("Mighty Blow", 1, false);
		assertEquals("Mighty Blow", m.getName());
		assertEquals(1, m.getModifier(null, null));
		assertFalse(m.isFoulAssistModifier());
	}

	// rust: chainsaw_flag_defaults_false
	@Test
	public void chainsawFlagDefaultsFalse() {
		assertFalse(new StaticArmourModifier("x", 0, false).isChainsaw());
	}

	// rust: chainsaw_flag_can_be_set
	@Test
	public void chainsawFlagCanBeSet() {
		StaticArmourModifier m = new StaticArmourModifier("Chainsaw +3", 3, false, true);
		assertTrue(m.isChainsaw());
	}

	// rust: set_registered_to_stores_value
	@Test
	public void setRegisteredToStoresValue() {
		StaticArmourModifier m = new StaticArmourModifier("x", 0, false);
		Skill dodge = skill("Dodge");
		m.setRegisteredTo(dodge);
		assertEquals(dodge, m.getRegisteredTo());
	}

	// rust: set_registered_to_then_clear_to_none
	@Test
	public void setRegisteredToThenClearToNone() {
		StaticArmourModifier m = new StaticArmourModifier("x", 0, false);
		m.setRegisteredTo(skill("Dodge"));
		m.setRegisteredTo(null);
		assertNull(m.getRegisteredTo());
	}
}
