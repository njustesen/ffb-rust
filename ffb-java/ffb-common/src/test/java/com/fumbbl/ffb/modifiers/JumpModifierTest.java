package com.fumbbl.ffb.modifiers;

import com.fumbbl.ffb.FieldCoordinate;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * 1:1 mirror of the Rust crates/ffb-mechanics/src/modifiers/jump_modifier.rs unit tests.
 * Java class: {@link com.fumbbl.ffb.modifiers.JumpModifier}.
 */
class JumpModifierTest {

	@Test
	void new_stores_fields() {
		JumpModifier m = new JumpModifier("1 Tacklezone", 1, ModifierType.TACKLEZONE);
		assertEquals("1 Tacklezone", m.getName());
		assertEquals(1, m.getModifier());
		assertEquals(ModifierType.TACKLEZONE, m.getType());
		assertEquals(1, m.getMultiplier());
	}

	@Test
	void new_multiplier_equals_modifier_not_hardcoded_one() {
		// 3-arg ctor delegates as this(name, name, pModifier, pModifier, type) —
		// multiplier tracks the modifier value, not a hardcoded 1.
		JumpModifier m = new JumpModifier("3 Tacklezones", 3, ModifierType.TACKLEZONE);
		assertEquals(3, m.getMultiplier());
	}

	@Test
	void tacklezone_is_modifier_included() {
		assertTrue(new JumpModifier("tz", 1, ModifierType.TACKLEZONE).isModifierIncluded());
	}

	@Test
	void regular_is_not_modifier_included() {
		assertFalse(new JumpModifier("x", 0, ModifierType.REGULAR).isModifierIncluded());
	}

	@Test
	void new_full_sets_multiplier_and_reporting_string() {
		// Rust new_full(name, report, modifier, multiplier, type) maps to the 5-arg Java constructor.
		JumpModifier m = new JumpModifier("Jump", "jump report", 1, 2, ModifierType.TACKLEZONE);
		assertEquals("Jump", m.getName());
		assertEquals("jump report", m.getReportString());
		assertEquals(2, m.getMultiplier());
		assertEquals(1, m.getModifier());
	}

	@Test
	void applies_to_context_without_predicate_returns_true() {
		JumpContext ctx = new JumpContext(null, null,
			new FieldCoordinate(0, 0), new FieldCoordinate(1, 1));
		JumpModifier m = new JumpModifier("x", 0, ModifierType.REGULAR);
		assertTrue(m.appliesToContext(null, ctx));
	}
}
