package com.fumbbl.ffb.modifiers;

import com.fumbbl.ffb.PassingDistance;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * 1:1 mirror of the Rust crates/ffb-mechanics/src/modifiers/pass_modifier.rs unit tests.
 * Java class: {@link com.fumbbl.ffb.modifiers.PassModifier}.
 */
class PassModifierTest {

	@Test
	void new_stores_fields() {
		PassModifier m = new PassModifier("Very Sunny", 1, ModifierType.REGULAR);
		assertEquals("Very Sunny", m.getName());
		assertEquals(1, m.getModifier());
		assertEquals(ModifierType.REGULAR, m.getType());
	}

	@Test
	void tacklezone_is_modifier_included() {
		assertTrue(new PassModifier("tz", 1, ModifierType.TACKLEZONE).isModifierIncluded());
	}

	@Test
	void regular_is_not_modifier_included() {
		assertFalse(new PassModifier("x", 0, ModifierType.REGULAR).isModifierIncluded());
	}

	@Test
	void applies_to_context_returns_true_without_predicate() {
		PassModifier m = new PassModifier("x", 1, ModifierType.REGULAR);
		PassContext ctx = new PassContext(null, null, PassingDistance.SHORT_PASS, false);
		assertTrue(m.appliesToContext(null, ctx));
	}

	@Test
	void with_report_stores_different_reporting_string() {
		// Rust with_report maps to the 4-arg (name, reportingString, modifier, type) constructor.
		PassModifier m = new PassModifier("Short Pass", "Short Pass Attempt", 0, ModifierType.REGULAR);
		assertEquals("Short Pass", m.getName());
		assertEquals("Short Pass Attempt", m.getReportString());
	}
}
