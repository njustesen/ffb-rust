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
 * Mirror of ffb-rust crates/ffb-mechanics/src/modifiers/variable_injury_modifier.rs tests.
 * VariableInjuryModifier is abstract in Java (Rust uses a use_attacker flag on the base struct);
 * the Rust new_attacker/new_defender factory fns map to the concrete Attacker/Defender subclasses.
 */
public class VariableInjuryModifierTest {

	private Skill skill(String name) {
		SkillFactory factory = NetCommandTestUtil.gameSource().getFactory(FactoryType.Factory.SKILL);
		return factory.forName(name);
	}

	// rust: new_attacker_stores_name_and_niggling_flag
	@Test
	public void newAttackerStoresNameAndNigglingFlag() {
		VariableInjuryModifierAttacker m = new VariableInjuryModifierAttacker("Mighty Blow", false);
		assertEquals("Mighty Blow", m.getName());
		assertFalse(m.isNigglingInjuryModifier());
	}

	// rust: new_defender_niggling_flag_propagates
	@Test
	public void newDefenderNigglingFlagPropagates() {
		VariableInjuryModifierDefender m = new VariableInjuryModifierDefender("Niggling", true);
		assertTrue(m.isNigglingInjuryModifier());
	}

	// rust: registered_to_defaults_none
	@Test
	public void registeredToDefaultsNone() {
		VariableInjuryModifierAttacker m = new VariableInjuryModifierAttacker("x", false);
		assertNull(m.getRegisteredTo());
	}

	// rust: set_registered_to_stores_value
	@Test
	public void setRegisteredToStoresValue() {
		VariableInjuryModifierAttacker m = new VariableInjuryModifierAttacker("x", false);
		Skill dodge = skill("Dodge");
		m.setRegisteredTo(dodge);
		assertEquals(dodge, m.getRegisteredTo());
	}

	// rust: set_registered_to_then_clear_to_none
	@Test
	public void setRegisteredToThenClearToNone() {
		VariableInjuryModifierAttacker m = new VariableInjuryModifierAttacker("x", false);
		m.setRegisteredTo(skill("Dodge"));
		m.setRegisteredTo(null);
		assertNull(m.getRegisteredTo());
	}
}
