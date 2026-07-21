package com.fumbbl.ffb.modifiers;

import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * 1:1 mirror of the Rust crates/ffb-mechanics/src/modifiers/jump_up_modifier.rs unit tests.
 * Java class: {@link com.fumbbl.ffb.modifiers.JumpUpModifier}.
 */
class JumpUpModifierTest {

	@Test
	void new_stores_fields() {
		JumpUpModifier m = new JumpUpModifier("Jump Up", -2, ModifierType.REGULAR);
		assertEquals("Jump Up", m.getName());
		assertEquals(-2, m.getModifier());
		assertEquals(ModifierType.REGULAR, m.getType());
	}

	@Test
	void is_modifier_included_always_false() {
		assertFalse(new JumpUpModifier("x", 0, ModifierType.REGULAR).isModifierIncluded());
		assertFalse(new JumpUpModifier("y", 1, ModifierType.TACKLEZONE).isModifierIncluded());
	}

	@Test
	void report_string_equals_name() {
		JumpUpModifier m = new JumpUpModifier("Jump Up", 0, ModifierType.REGULAR);
		assertEquals(m.getName(), m.getReportString());
	}

	@Test
	void get_type_returns_stored_type() {
		JumpUpModifier m = new JumpUpModifier("x", 0, ModifierType.TACKLEZONE);
		assertEquals(ModifierType.TACKLEZONE, m.getType());
	}

	@Test
	void applies_to_context_without_predicate_returns_true() {
		// Java JumpUpContext constructor is (ActingPlayer, Game); base appliesToContext ignores both.
		JumpUpContext ctx = new JumpUpContext(null, null);
		JumpUpModifier m = new JumpUpModifier("x", 0, ModifierType.REGULAR);
		assertTrue(m.appliesToContext(null, ctx));
	}
}
