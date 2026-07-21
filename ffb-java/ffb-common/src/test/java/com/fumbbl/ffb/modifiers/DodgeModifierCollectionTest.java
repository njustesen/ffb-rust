package com.fumbbl.ffb.modifiers;

import org.junit.jupiter.api.Test;

import java.util.Set;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirror of the Rust crates/ffb-mechanics/src/modifiers/dodge_modifier_collection.rs (abstract
 * base), modifiers/bb2016/dodge_modifier_collection.rs and modifiers/mixed/dodge_modifier_collection.rs
 * unit tests. The Rust base collection is a concrete struct; the Java base
 * {@link com.fumbbl.ffb.modifiers.DodgeModifierCollection} is abstract, so its 8 TACKLEZONE
 * base modifiers are asserted via a concrete subclass filtered to the TACKLEZONE type.
 */
class DodgeModifierCollectionTest {

	// ── abstract base (8 TACKLEZONE modifiers) ────────────────────────────────

	@Test
	void base_has_eight_tacklezone_modifiers() {
		com.fumbbl.ffb.modifiers.DodgeModifierCollection collection =
			new com.fumbbl.ffb.modifiers.mixed.DodgeModifierCollection();
		assertEquals(8, collection.getModifiers(ModifierType.TACKLEZONE).size());
	}

	@Test
	void base_includes_single_tacklezone_modifier() {
		com.fumbbl.ffb.modifiers.DodgeModifierCollection collection =
			new com.fumbbl.ffb.modifiers.mixed.DodgeModifierCollection();
		assertTrue(collection.getModifiers(ModifierType.TACKLEZONE).stream()
			.anyMatch(m -> "1 Tacklezone".equals(m.getName())));
	}

	@Test
	void base_all_tacklezone_modifiers_are_tacklezone_type() {
		com.fumbbl.ffb.modifiers.DodgeModifierCollection collection =
			new com.fumbbl.ffb.modifiers.mixed.DodgeModifierCollection();
		assertTrue(collection.getModifiers(ModifierType.TACKLEZONE).stream()
			.allMatch(m -> m.getType() == ModifierType.TACKLEZONE));
	}

	// ── mixed (BB2020/BB2025): base 8 tacklezone + 8 prehensile tail ──────────

	@Test
	void mixed_has_sixteen_modifiers() {
		assertEquals(16, new com.fumbbl.ffb.modifiers.mixed.DodgeModifierCollection().getModifiers().size());
	}

	@Test
	void mixed_includes_prehensile_tail_modifier() {
		Set<DodgeModifier> modifiers = new com.fumbbl.ffb.modifiers.mixed.DodgeModifierCollection().getModifiers();
		assertTrue(modifiers.stream().anyMatch(m -> "1 Prehensile Tail".equals(m.getName())));
	}

	@Test
	void mixed_plural_prehensile_tails_for_count_above_one() {
		Set<DodgeModifier> modifiers = new com.fumbbl.ffb.modifiers.mixed.DodgeModifierCollection().getModifiers();
		assertTrue(modifiers.stream().anyMatch(m -> "3 Prehensile Tails".equals(m.getName())));
	}

	@Test
	void mixed_all_modifiers_have_nonempty_names() {
		Set<DodgeModifier> modifiers = new com.fumbbl.ffb.modifiers.mixed.DodgeModifierCollection().getModifiers();
		assertTrue(modifiers.stream().allMatch(m -> m.getName() != null && !m.getName().isEmpty()));
	}

	// ── bb2016: base 8 tacklezone + 8 prehensile tail (scaling modifier) ──────

	@Test
	void bb2016_has_sixteen_modifiers() {
		assertEquals(16, new com.fumbbl.ffb.modifiers.bb2016.DodgeModifierCollection().getModifiers().size());
	}

	@Test
	void bb2016_includes_prehensile_tail_modifier() {
		Set<DodgeModifier> modifiers = new com.fumbbl.ffb.modifiers.bb2016.DodgeModifierCollection().getModifiers();
		assertTrue(modifiers.stream().anyMatch(m -> "1 Prehensile Tail".equals(m.getName())));
	}

	@Test
	void bb2016_plural_prehensile_tails_for_count_above_one() {
		Set<DodgeModifier> modifiers = new com.fumbbl.ffb.modifiers.bb2016.DodgeModifierCollection().getModifiers();
		assertTrue(modifiers.stream().anyMatch(m -> "2 Prehensile Tails".equals(m.getName())));
	}

	@Test
	void bb2016_includes_eight_tacklezone_modifier() {
		Set<DodgeModifier> modifiers = new com.fumbbl.ffb.modifiers.bb2016.DodgeModifierCollection().getModifiers();
		assertTrue(modifiers.stream().anyMatch(m -> "8 Tacklezones".equals(m.getName())));
	}

	@Test
	void bb2016_all_modifiers_have_nonempty_names() {
		Set<DodgeModifier> modifiers = new com.fumbbl.ffb.modifiers.bb2016.DodgeModifierCollection().getModifiers();
		assertTrue(modifiers.stream().allMatch(m -> m.getName() != null && !m.getName().isEmpty()));
	}

	/**
	 * Regression: BB2016's DodgeModifierCollection uses the 3-arg DodgeModifier(name, modifier,
	 * type) constructor, which sets modifier == multiplier, so the Prehensile Tail penalty scales
	 * with marker count (unlike the mixed BB2020/2025 collection which keeps a flat modifier of 1).
	 */
	@Test
	void bb2016_prehensile_tail_modifier_scales_with_marker_count() {
		Set<DodgeModifier> modifiers = new com.fumbbl.ffb.modifiers.bb2016.DodgeModifierCollection().getModifiers();
		DodgeModifier three = modifiers.stream()
			.filter(m -> "3 Prehensile Tails".equals(m.getName())).findFirst().orElseThrow();
		assertEquals(3, three.getModifier(), "BB2016 Prehensile Tail penalty must scale with marker count");
		assertEquals(3, three.getMultiplier());

		DodgeModifier one = modifiers.stream()
			.filter(m -> "1 Prehensile Tail".equals(m.getName())).findFirst().orElseThrow();
		assertEquals(1, one.getModifier());
	}
}
