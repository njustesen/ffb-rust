package com.fumbbl.ffb.modifiers;

import com.fumbbl.ffb.mechanics.PassResult;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * 1:1 mirror of the Rust crates/ffb-mechanics/src/modifiers/interception_modifier.rs unit tests.
 * Java class: {@link com.fumbbl.ffb.modifiers.InterceptionModifier}.
 */
class InterceptionModifierTest {

	@Test
	void new_stores_fields() {
		InterceptionModifier m = new InterceptionModifier("Accurate Pass", -2, ModifierType.REGULAR);
		assertEquals("Accurate Pass", m.getName());
		assertEquals(-2, m.getModifier());
		assertEquals(ModifierType.REGULAR, m.getType());
		// 3-arg ctor delegates as this(name, name, pModifier, pModifier, type) — multiplier
		// equals the modifier value, not a hardcoded 1.
		assertEquals(-2, m.getMultiplier());
	}

	@Test
	void new_multiplier_equals_modifier_for_disturbing_presence() {
		InterceptionModifier m = new InterceptionModifier("3 Disturbing Presences", 3, ModifierType.DISTURBING_PRESENCE);
		assertEquals(3, m.getMultiplier());
	}

	@Test
	void tacklezone_is_modifier_included() {
		assertTrue(new InterceptionModifier("tz", 1, ModifierType.TACKLEZONE).isModifierIncluded());
	}

	@Test
	void regular_is_not_modifier_included() {
		assertFalse(new InterceptionModifier("x", 1, ModifierType.REGULAR).isModifierIncluded());
	}

	@Test
	void new_full_sets_multiplier_and_reporting_string() {
		// Rust new_full(name, report, modifier, multiplier, type) maps to the 5-arg Java constructor.
		InterceptionModifier m = new InterceptionModifier("Catch", "catch report", -1, 2, ModifierType.REGULAR);
		assertEquals("Catch", m.getName());
		assertEquals("catch report", m.getReportString());
		assertEquals(2, m.getMultiplier());
		assertEquals(-1, m.getModifier());
	}

	@Test
	void applies_to_context_without_predicate_returns_true() {
		InterceptionContext ctx = new InterceptionContext(null, null, PassResult.ACCURATE, false);
		InterceptionModifier m = new InterceptionModifier("x", 0, ModifierType.REGULAR);
		assertTrue(m.appliesToContext(null, ctx));
	}
}
