package com.fumbbl.ffb.modifiers;

import com.fumbbl.ffb.FieldCoordinate;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * 1:1 mirror of the Rust crates/ffb-mechanics/src/modifiers/dodge_modifier.rs unit tests.
 * Java class: {@link com.fumbbl.ffb.modifiers.DodgeModifier}.
 */
class DodgeModifierTest {

	@Test
	void new_stores_name_modifier_type() {
		DodgeModifier m = new DodgeModifier("1 Tacklezone", 1, ModifierType.TACKLEZONE);
		assertEquals("1 Tacklezone", m.getName());
		assertEquals(1, m.getModifier());
		assertEquals(ModifierType.TACKLEZONE, m.getType());
	}

	@Test
	void tacklezone_type_is_modifier_included() {
		assertTrue(new DodgeModifier("tz", 1, ModifierType.TACKLEZONE).isModifierIncluded());
	}

	@Test
	void prehensile_tail_is_modifier_included() {
		assertTrue(new DodgeModifier("pt", 1, ModifierType.PREHENSILE_TAIL).isModifierIncluded());
	}

	@Test
	void regular_type_is_not_modifier_included() {
		assertFalse(new DodgeModifier("rain", -1, ModifierType.REGULAR).isModifierIncluded());
	}

	@Test
	void new_full_sets_all_fields() {
		// Rust new_full(name, report, modifier, multiplier, type, useStrength) maps to the
		// 6-arg Java constructor.
		DodgeModifier m = new DodgeModifier("Dodge", "dodge report", -1, 3, ModifierType.TACKLEZONE, true);
		assertEquals("Dodge", m.getName());
		assertEquals("dodge report", m.getReportString());
		assertEquals(-1, m.getModifier());
		assertEquals(3, m.getMultiplier());
		assertTrue(m.isUseStrength());
	}

	@Test
	void applies_to_context_without_predicate_returns_true() {
		DodgeContext ctx = new DodgeContext(null, null,
			new FieldCoordinate(0, 0), new FieldCoordinate(1, 1));
		DodgeModifier m = new DodgeModifier("x", 0, ModifierType.REGULAR);
		assertTrue(m.appliesToContext(null, ctx));
	}
}
