package com.fumbbl.ffb.modifiers;

import org.junit.jupiter.api.Test;

import java.util.Set;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirror of the Rust crates/ffb-mechanics/src/modifiers/go_for_it_modifier_collection.rs (base),
 * modifiers/bb2025/go_for_it_modifier_collection.rs and modifiers/mixed/go_for_it_modifier_collection.rs
 * unit tests. The Rust base collection is an empty, mutable struct exercised via add(); the Java base
 * {@link com.fumbbl.ffb.modifiers.GoForItModifierCollection} is abstract with no seeded content and a
 * protected add(), so the base tests that assert add()/len() mechanics on an empty collection and the
 * default()==new() test are not expressible and are skipped. The base name/value/type assertions are
 * mirrored against a directly constructed {@link GoForItModifier} (same real class/methods, same values).
 */
class GoForItModifierCollectionTest {

	// ── base: modifier contract asserted directly (empty mutable base is abstract in Java) ────────

	@Test
	void base_modifier_name_accessible() {
		assertEquals("Blizzard", new GoForItModifier("Blizzard", -1).getName());
	}

	@Test
	void base_modifier_value_accessible() {
		assertEquals(-1, new GoForItModifier("Blizzard", -1).getModifier());
	}

	@Test
	void base_modifier_type_is_regular() {
		assertEquals(ModifierType.REGULAR, new GoForItModifier("Blizzard", -1).getType());
		assertEquals(ModifierType.REGULAR, new GoForItModifier("Other", -2).getType());
	}

	// ── bb2025: blizzard + moles_under_pitch = 2 ─────────────────────────────────────────────────

	@Test
	void bb2025_has_two_modifiers() {
		assertEquals(2, new com.fumbbl.ffb.modifiers.bb2025.GoForItModifierCollection().getModifiers().size());
	}

	@Test
	void bb2025_includes_blizzard_modifier() {
		Set<GoForItModifier> modifiers = new com.fumbbl.ffb.modifiers.bb2025.GoForItModifierCollection().getModifiers();
		assertTrue(modifiers.stream().anyMatch(m -> "Blizzard".equals(m.getName())));
	}

	@Test
	void bb2025_includes_moles_modifier() {
		Set<GoForItModifier> modifiers = new com.fumbbl.ffb.modifiers.bb2025.GoForItModifierCollection().getModifiers();
		assertTrue(modifiers.stream().anyMatch(m -> "Moles under the Pitch".equals(m.getName())));
	}

	@Test
	void bb2025_blizzard_modifier_value_is_one() {
		Set<GoForItModifier> modifiers = new com.fumbbl.ffb.modifiers.bb2025.GoForItModifierCollection().getModifiers();
		GoForItModifier blizzard = modifiers.stream()
			.filter(m -> "Blizzard".equals(m.getName())).findFirst().orElseThrow();
		assertEquals(1, blizzard.getModifier());
	}

	@Test
	void bb2025_moles_modifier_value_is_one() {
		Set<GoForItModifier> modifiers = new com.fumbbl.ffb.modifiers.bb2025.GoForItModifierCollection().getModifiers();
		GoForItModifier moles = modifiers.stream()
			.filter(m -> "Moles under the Pitch".equals(m.getName())).findFirst().orElseThrow();
		assertEquals(1, moles.getModifier());
	}

	// ── mixed: blizzard + moles_home + moles_away = 3 ────────────────────────────────────────────

	@Test
	void mixed_has_three_modifiers() {
		assertEquals(3, new com.fumbbl.ffb.modifiers.mixed.GoForItModifierCollection().getModifiers().size());
	}

	@Test
	void mixed_includes_blizzard_modifier() {
		Set<GoForItModifier> modifiers = new com.fumbbl.ffb.modifiers.mixed.GoForItModifierCollection().getModifiers();
		assertTrue(modifiers.stream().anyMatch(m -> "Blizzard".equals(m.getName())));
	}

	@Test
	void mixed_has_moles_home_and_away() {
		Set<GoForItModifier> modifiers = new com.fumbbl.ffb.modifiers.mixed.GoForItModifierCollection().getModifiers();
		assertTrue(modifiers.stream().anyMatch(m -> "Moles under the Pitch (Home)".equals(m.getName())));
		assertTrue(modifiers.stream().anyMatch(m -> "Moles under the Pitch (Away)".equals(m.getName())));
	}

	@Test
	void mixed_all_modifiers_have_nonempty_names() {
		Set<GoForItModifier> modifiers = new com.fumbbl.ffb.modifiers.mixed.GoForItModifierCollection().getModifiers();
		assertTrue(modifiers.stream().allMatch(m -> m.getName() != null && !m.getName().isEmpty()));
	}
}
