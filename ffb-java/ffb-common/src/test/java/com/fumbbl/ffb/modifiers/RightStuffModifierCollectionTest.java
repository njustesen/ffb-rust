package com.fumbbl.ffb.modifiers;

import org.junit.jupiter.api.Test;

import java.util.Set;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirror of the Rust crates/ffb-mechanics/src/modifiers/right_stuff_modifier_collection.rs (base),
 * modifiers/bb2016, modifiers/bb2020 and modifiers/bb2025/right_stuff_modifier_collection.rs unit tests.
 * The Rust base collection is an empty, mutable struct exercised via add()/get_modifiers_by_type; the Java
 * base {@link com.fumbbl.ffb.modifiers.RightStuffModifierCollection} is abstract with a protected add(), so
 * base tests that assert add()/len()/get_modifiers_by_type mechanics on an empty collection and the
 * default()==new() test are not expressible and are skipped. The base modifier contract is mirrored against
 * a directly constructed {@link RightStuffModifier}.
 */
class RightStuffModifierCollectionTest {

	// ── base: modifier contract asserted directly (empty mutable base is abstract in Java) ────────

	@Test
	void base_modifier_construction() {
		RightStuffModifier modifier = new RightStuffModifier("test", 1, ModifierType.REGULAR);
		assertEquals("test", modifier.getName());
		assertEquals(1, modifier.getModifier());
		assertEquals(ModifierType.REGULAR, modifier.getType());
	}

	// ── bb2016: 2 kick range (medium + long) + 8 tacklezone = 10 ─────────────────────────────────

	@Test
	void bb2016_has_ten_modifiers() {
		assertEquals(10, new com.fumbbl.ffb.modifiers.bb2016.RightStuffModifierCollection().getModifiers().size());
	}

	@Test
	void bb2016_includes_medium_kick_modifier() {
		Set<RightStuffModifier> modifiers = new com.fumbbl.ffb.modifiers.bb2016.RightStuffModifierCollection().getModifiers();
		assertTrue(modifiers.stream().anyMatch(m -> "Medium Kick".equals(m.getName())));
	}

	@Test
	void bb2016_tacklezone_count_is_eight() {
		assertEquals(8, new com.fumbbl.ffb.modifiers.bb2016.RightStuffModifierCollection()
			.getModifiers(ModifierType.TACKLEZONE).size());
	}

	@Test
	void bb2016_includes_long_kick_modifier() {
		Set<RightStuffModifier> modifiers = new com.fumbbl.ffb.modifiers.bb2016.RightStuffModifierCollection().getModifiers();
		assertTrue(modifiers.stream().anyMatch(m -> "Long Kick".equals(m.getName())));
	}

	@Test
	void bb2016_long_kick_is_regular_type() {
		Set<RightStuffModifier> modifiers = new com.fumbbl.ffb.modifiers.bb2016.RightStuffModifierCollection().getModifiers();
		RightStuffModifier lk = modifiers.stream()
			.filter(m -> "Long Kick".equals(m.getName())).findFirst().orElseThrow();
		assertEquals(ModifierType.REGULAR, lk.getType());
	}

	// ── bb2020: 3 pass result (inaccurate/terrible/fumbled) + 8 tacklezone = 11 ──────────────────

	@Test
	void bb2020_has_eleven_modifiers() {
		assertEquals(11, new com.fumbbl.ffb.modifiers.bb2020.RightStuffModifierCollection().getModifiers().size());
	}

	@Test
	void bb2020_includes_successful_throw_modifier() {
		Set<RightStuffModifier> modifiers = new com.fumbbl.ffb.modifiers.bb2020.RightStuffModifierCollection().getModifiers();
		assertTrue(modifiers.stream().anyMatch(m -> "Successful Throw".equals(m.getName())));
	}

	@Test
	void bb2020_tacklezone_count_is_eight() {
		assertEquals(8, new com.fumbbl.ffb.modifiers.bb2020.RightStuffModifierCollection()
			.getModifiers(ModifierType.TACKLEZONE).size());
	}

	@Test
	void bb2020_includes_terrible_throw_modifier() {
		Set<RightStuffModifier> modifiers = new com.fumbbl.ffb.modifiers.bb2020.RightStuffModifierCollection().getModifiers();
		assertTrue(modifiers.stream().anyMatch(m -> "Terrible Throw".equals(m.getName())));
	}

	@Test
	void bb2020_includes_fumbled_throw_modifier() {
		Set<RightStuffModifier> modifiers = new com.fumbbl.ffb.modifiers.bb2020.RightStuffModifierCollection().getModifiers();
		assertTrue(modifiers.stream().anyMatch(m -> "Fumbled Throw".equals(m.getName())));
	}

	// ── bb2025: 2 pass result (subpar + fumbled) + 8 tacklezone = 10 ─────────────────────────────

	@Test
	void bb2025_has_ten_modifiers() {
		assertEquals(10, new com.fumbbl.ffb.modifiers.bb2025.RightStuffModifierCollection().getModifiers().size());
	}

	@Test
	void bb2025_includes_subpar_throw_modifier() {
		Set<RightStuffModifier> modifiers = new com.fumbbl.ffb.modifiers.bb2025.RightStuffModifierCollection().getModifiers();
		assertTrue(modifiers.stream().anyMatch(m -> "Subpar Throw".equals(m.getName())));
	}

	@Test
	void bb2025_includes_fumbled_throw_modifier() {
		Set<RightStuffModifier> modifiers = new com.fumbbl.ffb.modifiers.bb2025.RightStuffModifierCollection().getModifiers();
		assertTrue(modifiers.stream().anyMatch(m -> "Fumbled Throw".equals(m.getName())));
	}

	@Test
	void bb2025_tacklezone_count_is_eight() {
		assertEquals(8, new com.fumbbl.ffb.modifiers.bb2025.RightStuffModifierCollection()
			.getModifiers(ModifierType.TACKLEZONE).size());
	}

	@Test
	void bb2025_regular_type_count_is_two() {
		assertEquals(2, new com.fumbbl.ffb.modifiers.bb2025.RightStuffModifierCollection()
			.getModifiers(ModifierType.REGULAR).size());
	}
}
