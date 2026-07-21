package com.fumbbl.ffb.modifiers;

import org.junit.jupiter.api.Test;

import java.util.Set;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirror of the Rust crates/ffb-mechanics/src/modifiers/pass_modifier_collection.rs (abstract
 * base), modifiers/bb2016/pass_modifier_collection.rs and modifiers/mixed/pass_modifier_collection.rs
 * unit tests.
 *
 * <p>The Rust base collection is a concrete struct; the Java base
 * {@link com.fumbbl.ffb.modifiers.PassModifierCollection} is abstract, so its base modifiers
 * (1 "Very Sunny" REGULAR + 8 TACKLEZONE + 11 DISTURBING_PRESENCE = 20) are asserted via the
 * mixed concrete subclass, which adds no further modifiers and is therefore exactly the base.</p>
 */
class PassModifierCollectionTest {

	// ── abstract base (asserted via mixed, which == base with no additions) ────────

	@Test
	void base_initializes_with_base_modifiers() {
		assertTrue(!new com.fumbbl.ffb.modifiers.mixed.PassModifierCollection().getModifiers().isEmpty());
	}

	@Test
	void base_tacklezone_returns_eight_entries() {
		assertEquals(8, new com.fumbbl.ffb.modifiers.mixed.PassModifierCollection()
			.getModifiers(ModifierType.TACKLEZONE).size());
	}

	@Test
	void base_disturbing_presence_returns_eleven_entries() {
		assertEquals(11, new com.fumbbl.ffb.modifiers.mixed.PassModifierCollection()
			.getModifiers(ModifierType.DISTURBING_PRESENCE).size());
	}

	// ── mixed (BB2020/BB2025): base only, 20 modifiers ────────────────────────────

	@Test
	void mixed_has_twenty_modifiers() {
		assertEquals(20, new com.fumbbl.ffb.modifiers.mixed.PassModifierCollection().getModifiers().size());
	}

	@Test
	void mixed_includes_very_sunny_modifier() {
		Set<PassModifier> modifiers = new com.fumbbl.ffb.modifiers.mixed.PassModifierCollection().getModifiers();
		assertTrue(modifiers.stream().anyMatch(m -> "Very Sunny".equals(m.getName())));
	}

	@Test
	void mixed_includes_single_tacklezone_modifier() {
		Set<PassModifier> modifiers = new com.fumbbl.ffb.modifiers.mixed.PassModifierCollection().getModifiers();
		assertTrue(modifiers.stream().anyMatch(m -> "1 Tacklezone".equals(m.getName())));
	}

	@Test
	void mixed_includes_disturbing_presence_modifier() {
		Set<PassModifier> modifiers = new com.fumbbl.ffb.modifiers.mixed.PassModifierCollection().getModifiers();
		assertTrue(modifiers.stream().anyMatch(m -> m.getName().contains("Disturbing")));
	}

	@Test
	void mixed_all_modifiers_have_nonempty_names() {
		Set<PassModifier> modifiers = new com.fumbbl.ffb.modifiers.mixed.PassModifierCollection().getModifiers();
		assertTrue(modifiers.stream().allMatch(m -> m.getName() != null && !m.getName().isEmpty()));
	}

	// ── bb2016: base 20 + Blizzard = 21 ──────────────────────────────────────────

	@Test
	void bb2016_has_twenty_one_modifiers() {
		assertEquals(21, new com.fumbbl.ffb.modifiers.bb2016.PassModifierCollection().getModifiers().size());
	}

	@Test
	void bb2016_includes_blizzard_modifier() {
		Set<PassModifier> modifiers = new com.fumbbl.ffb.modifiers.bb2016.PassModifierCollection().getModifiers();
		assertTrue(modifiers.stream().anyMatch(m -> "Blizzard".equals(m.getName())));
	}

	@Test
	void bb2016_blizzard_is_regular_type() {
		PassModifier blizzard = new com.fumbbl.ffb.modifiers.bb2016.PassModifierCollection().getModifiers().stream()
			.filter(m -> "Blizzard".equals(m.getName())).findFirst().orElseThrow();
		assertEquals(ModifierType.REGULAR, blizzard.getType());
	}

	@Test
	void bb2016_includes_very_sunny_modifier() {
		Set<PassModifier> modifiers = new com.fumbbl.ffb.modifiers.bb2016.PassModifierCollection().getModifiers();
		assertTrue(modifiers.stream().anyMatch(m -> "Very Sunny".equals(m.getName())));
	}

	@Test
	void bb2016_very_sunny_is_regular_type() {
		PassModifier verySunny = new com.fumbbl.ffb.modifiers.bb2016.PassModifierCollection().getModifiers().stream()
			.filter(m -> "Very Sunny".equals(m.getName())).findFirst().orElseThrow();
		assertEquals(ModifierType.REGULAR, verySunny.getType());
	}
}
