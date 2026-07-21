package com.fumbbl.ffb.modifiers;

import com.fumbbl.ffb.CatchScatterThrowInMode;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * 1:1 mirror of the Rust crates/ffb-mechanics/src/modifiers/catch_modifier.rs unit tests.
 * Java class: {@link com.fumbbl.ffb.modifiers.CatchModifier}.
 */
class CatchModifierTest {

	@Test
	void new_stores_name_and_modifier() {
		CatchModifier m = new CatchModifier("Rain", -1, ModifierType.REGULAR);
		assertEquals("Rain", m.getName());
		assertEquals(-1, m.getModifier());
		assertEquals(ModifierType.REGULAR, m.getType());
	}

	@Test
	void tacklezone_type_is_modifier_included() {
		CatchModifier m = new CatchModifier("1 Tacklezone", 1, ModifierType.TACKLEZONE);
		assertTrue(m.isModifierIncluded());
	}

	@Test
	void regular_type_is_not_modifier_included() {
		CatchModifier m = new CatchModifier("Rain", -1, ModifierType.REGULAR);
		assertFalse(m.isModifierIncluded());
	}

	@Test
	void new_with_reporting_uses_separate_names() {
		// Rust new_with_reporting maps to the 4-arg (name, reportingString, modifier, type) constructor.
		CatchModifier m = new CatchModifier("internal", "display", 2, ModifierType.REGULAR);
		assertEquals("internal", m.getName());
		assertEquals("display", m.getReportString());
		assertEquals(2, m.getModifier());
	}

	@Test
	void applies_to_context_without_predicate_returns_true() {
		CatchContext ctx = new CatchContext(null, null,
			CatchScatterThrowInMode.CATCH_ACCURATE_PASS, null);
		CatchModifier m = new CatchModifier("Rain", -1, ModifierType.REGULAR);
		assertTrue(m.appliesToContext(null, ctx));
	}
}
