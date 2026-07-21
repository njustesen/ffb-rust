package com.fumbbl.ffb.modifiers;

import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * 1:1 mirror of the Rust crates/ffb-mechanics/src/modifiers/pickup_modifier.rs unit tests.
 * Java class: {@link com.fumbbl.ffb.modifiers.PickupModifier}.
 */
class PickupModifierTest {

	@Test
	void new_stores_fields() {
		PickupModifier m = new PickupModifier("1 Tacklezone", 1, ModifierType.TACKLEZONE);
		assertEquals("1 Tacklezone", m.getName());
		assertEquals(1, m.getModifier());
		assertEquals(ModifierType.TACKLEZONE, m.getType());
	}

	@Test
	void tacklezone_is_modifier_included() {
		assertTrue(new PickupModifier("tz", 1, ModifierType.TACKLEZONE).isModifierIncluded());
	}

	@Test
	void regular_is_not_modifier_included() {
		assertFalse(new PickupModifier("x", 0, ModifierType.REGULAR).isModifierIncluded());
	}

	@Test
	void applies_to_context_returns_true_without_predicate() {
		PickupModifier m = new PickupModifier("x", 1, ModifierType.REGULAR);
		PickupContext ctx = new PickupContext(null, null);
		assertTrue(m.appliesToContext(null, ctx));
	}

	@Test
	void new_full_uses_separate_reporting_string() {
		// Rust new_full maps to the 4-arg (name, reportString, modifier, type) constructor.
		PickupModifier m = new PickupModifier("Pick Up", "Picking Up", 1, ModifierType.REGULAR);
		assertEquals("Pick Up", m.getName());
		assertEquals("Picking Up", m.getReportString());
	}
}
