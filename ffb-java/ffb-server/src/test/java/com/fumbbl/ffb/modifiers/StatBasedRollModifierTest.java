package com.fumbbl.ffb.modifiers;

import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertNull;

/**
 * Mirror of ffb-rust crates/ffb-mechanics/src/modifiers/stat_based_roll_modifier.rs tests.
 */
public class StatBasedRollModifierTest {

	// rust: new_stores_name_and_value
	@Test
	public void newStoresNameAndValue() {
		StatBasedRollModifier m = new StatBasedRollModifier("Strength Bonus", 2);
		assertEquals("Strength Bonus", m.getReportString());
		assertEquals(2, m.getModifier());
	}

	// rust: is_modifier_included_always_false
	@Test
	public void isModifierIncludedAlwaysFalse() {
		assertFalse(new StatBasedRollModifier("x", 5).isModifierIncluded());
	}

	// rust: default_is_zero_empty
	// NOTE: Rust's Default uses an empty string for name; Java's no-arg ctor leaves it null (Rust
	// guards nulls as empty strings — documented divergence). Assert Java's actual behavior.
	@Test
	public void defaultIsZeroEmpty() {
		StatBasedRollModifier m = new StatBasedRollModifier();
		assertEquals(0, m.getModifier());
		assertNull(m.getName());
	}

	// rust: two_modifiers_with_same_fields_are_equal
	@Test
	public void twoModifiersWithSameFieldsAreEqual() {
		assertEquals(new StatBasedRollModifier("Str", 3), new StatBasedRollModifier("Str", 3));
	}
}
