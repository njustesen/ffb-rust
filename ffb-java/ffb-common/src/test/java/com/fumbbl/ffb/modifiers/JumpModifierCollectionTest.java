package com.fumbbl.ffb.modifiers;

import org.junit.jupiter.api.Test;

import java.util.Set;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirror of the Rust crates/ffb-mechanics/src/modifiers/jump_modifier_collection.rs (base),
 * modifiers/bb2016/jump_modifier_collection.rs and modifiers/mixed/jump_modifier_collection.rs
 * unit tests. The Rust base collection is an empty, mutable struct exercised via add(); the Java base
 * {@link com.fumbbl.ffb.modifiers.JumpModifierCollection} is abstract with a protected add(), so base
 * tests that assert add()/len()/get_modifiers_by_type mechanics on an empty collection and the
 * default()==new() test are not expressible and are skipped. The base name/value assertions are mirrored
 * against a directly constructed {@link JumpModifier}. The bb2016 find_applicable test is skipped: Java's
 * ModifierCollection has no find_applicable method (and it requires heavy Game/Player model setup).
 */
class JumpModifierCollectionTest {

	// ── base: modifier contract asserted directly (empty mutable base is abstract in Java) ────────

	@Test
	void base_modifier_name_accessible() {
		assertEquals("1 Tacklezone", new JumpModifier("1 Tacklezone", 1, ModifierType.TACKLEZONE).getName());
	}

	@Test
	void base_modifier_value_accessible() {
		assertEquals(-1, new JumpModifier("TZ", -1, ModifierType.TACKLEZONE).getModifier());
	}

	// ── bb2016: no extra jump modifiers on top of empty base = 0 ─────────────────────────────────

	@Test
	void bb2016_has_zero_modifiers() {
		assertEquals(0, new com.fumbbl.ffb.modifiers.bb2016.JumpModifierCollection().getModifiers().size());
	}

	@Test
	void bb2016_get_modifiers_returns_empty() {
		assertTrue(new com.fumbbl.ffb.modifiers.bb2016.JumpModifierCollection().getModifiers().isEmpty());
	}

	@Test
	void bb2016_all_modifiers_have_nonempty_names() {
		Set<JumpModifier> modifiers = new com.fumbbl.ffb.modifiers.bb2016.JumpModifierCollection().getModifiers();
		assertTrue(modifiers.stream().allMatch(m -> m.getName() != null && !m.getName().isEmpty()));
	}

	// ── mixed: 8 prehensile_tail + 8 tacklezone = 16 ─────────────────────────────────────────────

	@Test
	void mixed_has_sixteen_modifiers() {
		assertEquals(16, new com.fumbbl.ffb.modifiers.mixed.JumpModifierCollection().getModifiers().size());
	}

	@Test
	void mixed_includes_prehensile_tail_modifier() {
		Set<JumpModifier> modifiers = new com.fumbbl.ffb.modifiers.mixed.JumpModifierCollection().getModifiers();
		assertTrue(modifiers.stream().anyMatch(m -> "1 Prehensile Tail".equals(m.getName())));
	}

	@Test
	void mixed_includes_tacklezone_modifier() {
		Set<JumpModifier> modifiers = new com.fumbbl.ffb.modifiers.mixed.JumpModifierCollection().getModifiers();
		assertTrue(modifiers.stream().anyMatch(m -> "1 Tacklezone".equals(m.getName())));
	}

	@Test
	void mixed_all_modifiers_have_nonempty_names() {
		Set<JumpModifier> modifiers = new com.fumbbl.ffb.modifiers.mixed.JumpModifierCollection().getModifiers();
		assertTrue(modifiers.stream().allMatch(m -> m.getName() != null && !m.getName().isEmpty()));
	}
}
