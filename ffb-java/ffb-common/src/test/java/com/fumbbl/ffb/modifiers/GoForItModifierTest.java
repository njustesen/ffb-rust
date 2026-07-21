package com.fumbbl.ffb.modifiers;

import org.junit.jupiter.api.Test;

import java.util.Collections;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * 1:1 mirror of the Rust crates/ffb-mechanics/src/modifiers/go_for_it_modifier.rs unit tests.
 * Java class: {@link com.fumbbl.ffb.modifiers.GoForItModifier}.
 */
class GoForItModifierTest {

	@Test
	void new_stores_name_and_modifier() {
		GoForItModifier m = new GoForItModifier("Blizzard", -1);
		assertEquals("Blizzard", m.getName());
		assertEquals(-1, m.getModifier());
	}

	@Test
	void type_is_always_regular() {
		assertEquals(ModifierType.REGULAR, new GoForItModifier("x", 0).getType());
	}

	@Test
	void is_modifier_included_always_false() {
		assertFalse(new GoForItModifier("x", 0).isModifierIncluded());
	}

	@Test
	void report_string_equals_name() {
		GoForItModifier m = new GoForItModifier("Blizzard", -1);
		assertEquals(m.getName(), m.getReportString());
	}

	@Test
	void applies_to_context_without_predicate_returns_true() {
		GoForItContext ctx = new GoForItContext(null, null, Collections.emptySet());
		GoForItModifier m = new GoForItModifier("x", 0);
		assertTrue(m.appliesToContext(null, ctx));
	}
}
