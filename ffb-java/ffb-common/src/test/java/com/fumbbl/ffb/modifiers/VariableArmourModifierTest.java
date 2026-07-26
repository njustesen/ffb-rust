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
 * Mirror of ffb-rust crates/ffb-mechanics/src/modifiers/variable_armour_modifier.rs tests.
 * The Rust with_modifier_fn / with_predicate builder tests are Rust-only (Java getModifier is
 * fixed to attacker.getSkillIntValue(registeredTo) and appliesToContext always true) — exempt.
 */
public class VariableArmourModifierTest {

	private Skill skill(String name) {
		SkillFactory factory = NetCommandTestUtil.gameSource().getFactory(FactoryType.Factory.SKILL);
		return factory.forName(name);
	}

	// rust: new_stores_name_and_foul_flag
	@Test
	public void newStoresNameAndFoulFlag() {
		VariableArmourModifier m = new VariableArmourModifier("Mighty Blow", false);
		assertEquals("Mighty Blow", m.getName());
		assertFalse(m.isFoulAssistModifier());
	}

	// rust: registered_to_defaults_none
	@Test
	public void registeredToDefaultsNone() {
		VariableArmourModifier m = new VariableArmourModifier("x", false);
		assertNull(m.getRegisteredTo());
	}

	// rust: foul_assist_flag_propagates
	@Test
	public void foulAssistFlagPropagates() {
		VariableArmourModifier mFoul = new VariableArmourModifier("Foul Assist", true);
		assertTrue(mFoul.isFoulAssistModifier());
		VariableArmourModifier mNoFoul = new VariableArmourModifier("x", false);
		assertFalse(mNoFoul.isFoulAssistModifier());
	}

	// rust: set_registered_to_stores_value
	@Test
	public void setRegisteredToStoresValue() {
		VariableArmourModifier m = new VariableArmourModifier("x", false);
		Skill dodge = skill("Dodge");
		m.setRegisteredTo(dodge);
		assertEquals(dodge, m.getRegisteredTo());
	}

	// rust: set_registered_to_then_clear_to_none
	@Test
	public void setRegisteredToThenClearToNone() {
		VariableArmourModifier m = new VariableArmourModifier("x", false);
		m.setRegisteredTo(skill("Dodge"));
		m.setRegisteredTo(null);
		assertNull(m.getRegisteredTo());
	}
}
