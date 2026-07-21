package com.fumbbl.ffb.modifiers;

import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * 1:1 mirror of the Rust crates/ffb-mechanics/src/modifiers/gaze_modifier.rs unit tests.
 * Java class: {@link com.fumbbl.ffb.modifiers.GazeModifier}.
 */
class GazeModifierTest {

	@Test
	void new_stores_fields() {
		GazeModifier m = new GazeModifier("1 Tacklezone", 1, ModifierType.TACKLEZONE);
		assertEquals("1 Tacklezone", m.getName());
		assertEquals(1, m.getModifier());
		assertEquals(ModifierType.TACKLEZONE, m.getType());
	}

	@Test
	void tacklezone_is_modifier_included() {
		assertTrue(new GazeModifier("tz", 1, ModifierType.TACKLEZONE).isModifierIncluded());
	}

	@Test
	void regular_is_not_modifier_included() {
		assertFalse(new GazeModifier("x", 0, ModifierType.REGULAR).isModifierIncluded());
	}

	@Test
	void new_full_sets_multiplier() {
		// Rust new_full(name, report, modifier, multiplier, type) maps to the 5-arg Java constructor.
		GazeModifier m = new GazeModifier("Gaze", "Hypnotic Gaze", -1, 3, ModifierType.REGULAR);
		assertEquals("Gaze", m.getName());
		assertEquals("Hypnotic Gaze", m.getReportString());
		assertEquals(3, m.getMultiplier());
		assertEquals(-1, m.getModifier());
	}

	@Test
	void applies_to_context_without_predicate_returns_true() {
		GazeModifierContext ctx = new GazeModifierContext(null, null);
		GazeModifier m = new GazeModifier("x", 0, ModifierType.REGULAR);
		assertTrue(m.appliesToContext(null, ctx));
	}
}
