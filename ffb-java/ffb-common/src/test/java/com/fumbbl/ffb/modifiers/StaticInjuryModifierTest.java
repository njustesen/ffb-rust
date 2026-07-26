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
 * Mirror of ffb-rust crates/ffb-mechanics/src/modifiers/static_injury_modifier.rs tests.
 * Java setRegisteredTo takes a Skill (Rust uses a skill-id string); a real Dodge Skill is
 * resolved from the game-source SkillFactory to exercise the registration getters.
 */
public class StaticInjuryModifierTest {

	private Skill skill(String name) {
		SkillFactory factory = NetCommandTestUtil.gameSource().getFactory(FactoryType.Factory.SKILL);
		return factory.forName(name);
	}

	// rust: new_stores_fields
	@Test
	public void newStoresFields() {
		StaticInjuryModifier m = new StaticInjuryModifier("Stunty", 1, false);
		assertEquals("Stunty", m.getName());
		assertEquals(1, m.getModifier(null, null));
		assertFalse(m.isNigglingInjuryModifier());
	}

	// rust: niggling_flag_propagates
	@Test
	public void nigglingFlagPropagates() {
		StaticInjuryModifier m = new StaticInjuryModifier("Niggling", 0, true);
		assertTrue(m.isNigglingInjuryModifier());
	}

	// rust: registered_to_defaults_none
	@Test
	public void registeredToDefaultsNone() {
		StaticInjuryModifier m = new StaticInjuryModifier("x", 0, false);
		assertNull(m.getRegisteredTo());
	}

	// rust: set_registered_to_stores_value
	@Test
	public void setRegisteredToStoresValue() {
		StaticInjuryModifier m = new StaticInjuryModifier("x", 0, false);
		Skill dodge = skill("Dodge");
		m.setRegisteredTo(dodge);
		assertEquals(dodge, m.getRegisteredTo());
	}

	// rust: set_registered_to_then_clear_to_none
	@Test
	public void setRegisteredToThenClearToNone() {
		StaticInjuryModifier m = new StaticInjuryModifier("x", 0, false);
		m.setRegisteredTo(skill("Dodge"));
		m.setRegisteredTo(null);
		assertNull(m.getRegisteredTo());
	}
}
