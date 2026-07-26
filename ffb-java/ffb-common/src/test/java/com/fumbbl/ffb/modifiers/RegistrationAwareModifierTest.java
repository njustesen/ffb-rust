package com.fumbbl.ffb.modifiers;

import com.fumbbl.ffb.FactoryType;
import com.fumbbl.ffb.factory.SkillFactory;
import com.fumbbl.ffb.model.property.NamedProperties;
import com.fumbbl.ffb.model.skill.Skill;
import com.fumbbl.ffb.net.NetCommandTestUtil;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertNull;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirror of ffb-rust crates/ffb-mechanics/src/modifiers/registration_aware_modifier.rs tests.
 * RegistrationAwareModifier is abstract in Java (all methods concrete) — subclassed in-test as Stub.
 * isRegisteredToSkillWithProperty checks registeredTo != null && registeredTo.hasSkillProperty(p).
 */
public class RegistrationAwareModifierTest {

	private static class Stub extends RegistrationAwareModifier {
	}

	private Skill skill(String name) {
		SkillFactory factory = NetCommandTestUtil.gameSource().getFactory(FactoryType.Factory.SKILL);
		return factory.forName(name);
	}

	// rust: is_registered_to_skill_with_property_false_when_unregistered
	@Test
	public void isRegisteredToSkillWithPropertyFalseWhenUnregistered() {
		Stub m = new Stub();
		assertFalse(m.isRegisteredToSkillWithProperty(NamedProperties.canLeap));
	}

	// rust: is_registered_to_skill_with_property_true_for_leap_property
	@Test
	public void isRegisteredToSkillWithPropertyTrueForLeapProperty() {
		Stub m = new Stub();
		m.setRegisteredTo(skill("Leap"));
		assertTrue(m.isRegisteredToSkillWithProperty(NamedProperties.canLeap));
	}

	// rust: is_registered_to_skill_with_property_false_for_wrong_property
	@Test
	public void isRegisteredToSkillWithPropertyFalseForWrongProperty() {
		Stub m = new Stub();
		m.setRegisteredTo(skill("Leap"));
		assertFalse(m.isRegisteredToSkillWithProperty(NamedProperties.canAvoidFallingDown));
	}

	// rust: set_registered_to_stores_value
	@Test
	public void setRegisteredToStoresValue() {
		Stub m = new Stub();
		Skill dodge = skill("Dodge");
		m.setRegisteredTo(dodge);
		assertEquals(dodge, m.getRegisteredTo());
	}

	// rust: set_registered_to_then_clear_to_none
	@Test
	public void setRegisteredToThenClearToNone() {
		Stub m = new Stub();
		m.setRegisteredTo(skill("Dodge"));
		m.setRegisteredTo(null);
		assertNull(m.getRegisteredTo());
	}
}
