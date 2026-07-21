package com.fumbbl.ffb.modifiers;

import com.fumbbl.ffb.mechanics.PassResult;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * 1:1 mirror of the Rust crates/ffb-mechanics/src/modifiers/right_stuff_modifier.rs unit tests.
 * Java class: {@link com.fumbbl.ffb.modifiers.RightStuffModifier}.
 */
class RightStuffModifierTest {

	@Test
	void new_stores_fields() {
		RightStuffModifier m = new RightStuffModifier("Medium Kick", -1, ModifierType.REGULAR);
		assertEquals("Medium Kick", m.getName());
		assertEquals(-1, m.getModifier());
		assertEquals(ModifierType.REGULAR, m.getType());
	}

	@Test
	void tacklezone_is_modifier_included() {
		assertTrue(new RightStuffModifier("tz", 1, ModifierType.TACKLEZONE).isModifierIncluded());
	}

	@Test
	void regular_is_not_modifier_included() {
		assertFalse(new RightStuffModifier("x", 0, ModifierType.REGULAR).isModifierIncluded());
	}

	@Test
	void applies_to_context_returns_true_without_predicate() {
		RightStuffModifier m = new RightStuffModifier("x", 1, ModifierType.REGULAR);
		RightStuffContext ctx = new RightStuffContext(null, null, PassResult.ACCURATE);
		assertTrue(m.appliesToContext(null, ctx));
	}

	// SKIPPED: Rust new_full_uses_separate_reporting_string — the Java RightStuffModifier has only the
	// single 3-arg (name, modifier, type) constructor and getReportString() returns getName(); it has no
	// constructor accepting a separate reporting string, so the Rust new_full behaviour is inexpressible.
}
