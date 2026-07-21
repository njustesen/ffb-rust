package com.fumbbl.ffb.modifiers;

import org.junit.jupiter.api.Test;

import java.util.Set;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirror of the Rust crates/ffb-mechanics/src/modifiers/jump_up_modifier_collection.rs (base),
 * modifiers/bb2016/jump_up_modifier_collection.rs and modifiers/mixed/jump_up_modifier_collection.rs
 * unit tests. The Rust base collection is an empty, mutable struct exercised via add(); the Java base
 * {@link com.fumbbl.ffb.modifiers.JumpUpModifierCollection} is abstract with a protected add(), so base
 * tests that assert add()/len()/get_modifiers_by_type mechanics on an empty collection and the
 * default()==new() test are not expressible and are skipped. The base name/value assertions are mirrored
 * against a directly constructed {@link JumpUpModifier}.
 */
class JumpUpModifierCollectionTest {

	// ── base: modifier contract asserted directly (empty mutable base is abstract in Java) ────────

	@Test
	void base_modifier_name_accessible() {
		assertEquals("Jump Up", new JumpUpModifier("Jump Up", -2, ModifierType.REGULAR).getName());
	}

	@Test
	void base_modifier_value_accessible() {
		assertEquals(-2, new JumpUpModifier("Jump Up", -2, ModifierType.REGULAR).getModifier());
	}

	// ── bb2016: "Jump Up" (-2) = 1 modifier ──────────────────────────────────────────────────────

	@Test
	void bb2016_has_one_modifier() {
		assertEquals(1, new com.fumbbl.ffb.modifiers.bb2016.JumpUpModifierCollection().getModifiers().size());
	}

	@Test
	void bb2016_includes_jump_up_modifier() {
		Set<JumpUpModifier> modifiers = new com.fumbbl.ffb.modifiers.bb2016.JumpUpModifierCollection().getModifiers();
		assertTrue(modifiers.stream().anyMatch(m -> "Jump Up".equals(m.getName())));
	}

	@Test
	void bb2016_jump_up_modifier_is_regular_type() {
		Set<JumpUpModifier> modifiers = new com.fumbbl.ffb.modifiers.bb2016.JumpUpModifierCollection().getModifiers();
		JumpUpModifier ju = modifiers.stream()
			.filter(m -> "Jump Up".equals(m.getName())).findFirst().orElseThrow();
		assertEquals(ModifierType.REGULAR, ju.getType());
	}

	@Test
	void bb2016_jump_up_modifier_value_is_minus_two() {
		Set<JumpUpModifier> modifiers = new com.fumbbl.ffb.modifiers.bb2016.JumpUpModifierCollection().getModifiers();
		JumpUpModifier ju = modifiers.stream()
			.filter(m -> "Jump Up".equals(m.getName())).findFirst().orElseThrow();
		assertEquals(-2, ju.getModifier());
	}

	@Test
	void bb2016_regular_type_count_is_one() {
		assertEquals(1, new com.fumbbl.ffb.modifiers.bb2016.JumpUpModifierCollection()
			.getModifiers(ModifierType.REGULAR).size());
	}

	// ── mixed: "Jump Up" (-1) = 1 modifier ───────────────────────────────────────────────────────

	@Test
	void mixed_has_one_modifier() {
		assertEquals(1, new com.fumbbl.ffb.modifiers.mixed.JumpUpModifierCollection().getModifiers().size());
	}

	@Test
	void mixed_includes_jump_up_modifier() {
		Set<JumpUpModifier> modifiers = new com.fumbbl.ffb.modifiers.mixed.JumpUpModifierCollection().getModifiers();
		assertTrue(modifiers.stream().anyMatch(m -> "Jump Up".equals(m.getName())));
	}

	@Test
	void mixed_jump_up_is_regular_type() {
		Set<JumpUpModifier> modifiers = new com.fumbbl.ffb.modifiers.mixed.JumpUpModifierCollection().getModifiers();
		JumpUpModifier ju = modifiers.stream()
			.filter(m -> "Jump Up".equals(m.getName())).findFirst().orElseThrow();
		assertEquals(ModifierType.REGULAR, ju.getType());
	}

	@Test
	void mixed_jump_up_has_negative_one_modifier_value() {
		Set<JumpUpModifier> modifiers = new com.fumbbl.ffb.modifiers.mixed.JumpUpModifierCollection().getModifiers();
		JumpUpModifier ju = modifiers.stream()
			.filter(m -> "Jump Up".equals(m.getName())).findFirst().orElseThrow();
		assertEquals(-1, ju.getModifier());
	}

	@Test
	void mixed_all_modifiers_have_nonempty_names() {
		Set<JumpUpModifier> modifiers = new com.fumbbl.ffb.modifiers.mixed.JumpUpModifierCollection().getModifiers();
		assertTrue(modifiers.stream().allMatch(m -> m.getName() != null && !m.getName().isEmpty()));
	}
}
