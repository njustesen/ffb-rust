package com.fumbbl.ffb.modifiers;

import org.junit.jupiter.api.Test;

import java.util.Set;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirror of the Rust crates/ffb-mechanics/src/modifiers/catch_modifier_collection.rs (abstract
 * base), modifiers/bb2016/catch_modifier_collection.rs, modifiers/bb2020/catch_modifier_collection.rs
 * and modifiers/bb2025/catch_modifier_collection.rs unit tests.
 *
 * <p>The Rust base collection is a concrete struct; the Java base
 * {@link com.fumbbl.ffb.modifiers.CatchModifierCollection} is abstract, so its base modifiers
 * (8 TACKLEZONE + 11 DISTURBING_PRESENCE + 1 "Pouring Rain" REGULAR = 20) are asserted via a
 * concrete subclass filtered to the relevant ModifierType (mirroring how the reference
 * DodgeModifierCollectionTest handles the abstract base).</p>
 */
class CatchModifierCollectionTest {

	// ── abstract base (8 tacklezone + 11 disturbing presence + 1 pouring rain = 20) ──
	// The exact base count of 20 cannot be isolated on the abstract Java base because every
	// concrete subclass adds further REGULAR modifiers; the base is asserted via its stable
	// TACKLEZONE / DISTURBING_PRESENCE type counts plus the named base modifiers.

	@Test
	void base_has_eight_tacklezone_modifiers() {
		com.fumbbl.ffb.modifiers.CatchModifierCollection collection =
			new com.fumbbl.ffb.modifiers.bb2020.CatchModifierCollection();
		assertEquals(8, collection.getModifiers(ModifierType.TACKLEZONE).size());
	}

	@Test
	void base_has_eleven_disturbing_presence_modifiers() {
		com.fumbbl.ffb.modifiers.CatchModifierCollection collection =
			new com.fumbbl.ffb.modifiers.bb2020.CatchModifierCollection();
		assertEquals(11, collection.getModifiers(ModifierType.DISTURBING_PRESENCE).size());
	}

	@Test
	void base_includes_single_tacklezone_modifier() {
		com.fumbbl.ffb.modifiers.CatchModifierCollection collection =
			new com.fumbbl.ffb.modifiers.bb2020.CatchModifierCollection();
		assertTrue(collection.getModifiers().stream().anyMatch(m -> "1 Tacklezone".equals(m.getName())));
	}

	@Test
	void base_includes_pouring_rain_modifier() {
		com.fumbbl.ffb.modifiers.CatchModifierCollection collection =
			new com.fumbbl.ffb.modifiers.bb2020.CatchModifierCollection();
		assertTrue(collection.getModifiers().stream().anyMatch(m -> "Pouring Rain".equals(m.getName())));
	}

	@Test
	void base_all_modifiers_have_nonempty_names() {
		com.fumbbl.ffb.modifiers.CatchModifierCollection collection =
			new com.fumbbl.ffb.modifiers.bb2020.CatchModifierCollection();
		assertTrue(collection.getModifiers().stream().allMatch(m -> m.getName() != null && !m.getName().isEmpty()));
	}

	// ── bb2016: base 20 + Accurate Pass + Hand Off = 22 ──────────────────────────

	@Test
	void bb2016_has_twenty_two_modifiers() {
		assertEquals(22, new com.fumbbl.ffb.modifiers.bb2016.CatchModifierCollection().getModifiers().size());
	}

	@Test
	void bb2016_includes_accurate_pass_modifier() {
		Set<CatchModifier> modifiers = new com.fumbbl.ffb.modifiers.bb2016.CatchModifierCollection().getModifiers();
		assertTrue(modifiers.stream().anyMatch(m -> "Accurate Pass".equals(m.getName())));
	}

	@Test
	void bb2016_includes_hand_off_modifier() {
		Set<CatchModifier> modifiers = new com.fumbbl.ffb.modifiers.bb2016.CatchModifierCollection().getModifiers();
		assertTrue(modifiers.stream().anyMatch(m -> "Hand Off".equals(m.getName())));
	}

	@Test
	void bb2016_disturbing_presence_count_is_eleven() {
		assertEquals(11, new com.fumbbl.ffb.modifiers.bb2016.CatchModifierCollection()
			.getModifiers(ModifierType.DISTURBING_PRESENCE).size());
	}

	@Test
	void bb2016_regular_type_count_includes_accurate_pass_and_hand_off() {
		// pouring_rain(1) + accurate_pass(1) + hand_off(1) = 3
		assertEquals(3, new com.fumbbl.ffb.modifiers.bb2016.CatchModifierCollection()
			.getModifiers(ModifierType.REGULAR).size());
	}

	// ── bb2020: base 20 + Inaccurate + Deflected Pass + Blast It! = 23 ────────────

	@Test
	void bb2020_has_twenty_three_modifiers() {
		assertEquals(23, new com.fumbbl.ffb.modifiers.bb2020.CatchModifierCollection().getModifiers().size());
	}

	@Test
	void bb2020_includes_deflected_pass_modifier() {
		Set<CatchModifier> modifiers = new com.fumbbl.ffb.modifiers.bb2020.CatchModifierCollection().getModifiers();
		assertTrue(modifiers.stream().anyMatch(m -> "Deflected Pass".equals(m.getName())));
	}

	@Test
	void bb2020_includes_blast_it_modifier() {
		Set<CatchModifier> modifiers = new com.fumbbl.ffb.modifiers.bb2020.CatchModifierCollection().getModifiers();
		assertTrue(modifiers.stream().anyMatch(m -> "Blast It!".equals(m.getName())));
	}

	@Test
	void bb2020_disturbing_presence_count_is_eleven() {
		assertEquals(11, new com.fumbbl.ffb.modifiers.bb2020.CatchModifierCollection()
			.getModifiers(ModifierType.DISTURBING_PRESENCE).size());
	}

	@Test
	void bb2020_tacklezone_count_is_eight() {
		assertEquals(8, new com.fumbbl.ffb.modifiers.bb2020.CatchModifierCollection()
			.getModifiers(ModifierType.TACKLEZONE).size());
	}

	// ── bb2025: base 20 + Inaccurate Pass or Scatter + Blast It! = 22 ─────────────

	@Test
	void bb2025_has_twenty_two_modifiers() {
		assertEquals(22, new com.fumbbl.ffb.modifiers.bb2025.CatchModifierCollection().getModifiers().size());
	}

	@Test
	void bb2025_includes_inaccurate_pass_or_scatter() {
		Set<CatchModifier> modifiers = new com.fumbbl.ffb.modifiers.bb2025.CatchModifierCollection().getModifiers();
		assertTrue(modifiers.stream().anyMatch(m -> "Inaccurate Pass or Scatter".equals(m.getName())));
	}

	@Test
	void bb2025_includes_blast_it_modifier() {
		Set<CatchModifier> modifiers = new com.fumbbl.ffb.modifiers.bb2025.CatchModifierCollection().getModifiers();
		assertTrue(modifiers.stream().anyMatch(m -> "Blast It!".equals(m.getName())));
	}

	@Test
	void bb2025_disturbing_presence_count_is_eleven() {
		assertEquals(11, new com.fumbbl.ffb.modifiers.bb2025.CatchModifierCollection()
			.getModifiers(ModifierType.DISTURBING_PRESENCE).size());
	}

	@Test
	void bb2025_tacklezone_count_is_eight() {
		assertEquals(8, new com.fumbbl.ffb.modifiers.bb2025.CatchModifierCollection()
			.getModifiers(ModifierType.TACKLEZONE).size());
	}
}
