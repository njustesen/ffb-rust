package com.fumbbl.ffb.modifiers;

import org.junit.jupiter.api.Test;

import java.util.Set;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirror of the Rust crates/ffb-mechanics/src/modifiers/interception_modifier_collection.rs (base),
 * modifiers/bb2016, modifiers/bb2020 and modifiers/bb2025/interception_modifier_collection.rs unit tests.
 * The Rust base collection is a concrete struct seeded with 11 disturbing-presence + 1 pouring-rain = 12
 * modifiers; the Java base {@link com.fumbbl.ffb.modifiers.InterceptionModifierCollection} is abstract but
 * its constructor seeds those same 12 modifiers. The edition subclasses only ever add TACKLEZONE/REGULAR
 * modifiers, so the base contribution is asserted via a concrete subclass (bb2016) filtered to the
 * DISTURBING_PRESENCE type plus the Pouring Rain modifier, mirroring the reference DodgeModifierCollection
 * pattern. The default()==new() test is Rust-only and skipped.
 */
class InterceptionModifierCollectionTest {

	// ── base: 11 disturbing_presence + 1 pouring_rain = 12 (asserted via concrete subclass) ──────

	@Test
	void base_has_twelve_base_modifiers() {
		com.fumbbl.ffb.modifiers.InterceptionModifierCollection collection =
			new com.fumbbl.ffb.modifiers.bb2016.InterceptionModifierCollection();
		assertEquals(11, collection.getModifiers(ModifierType.DISTURBING_PRESENCE).size());
		assertTrue(collection.getModifiers().stream().anyMatch(m -> "Pouring Rain".equals(m.getName())));
	}

	@Test
	void base_includes_pouring_rain_modifier() {
		com.fumbbl.ffb.modifiers.InterceptionModifierCollection collection =
			new com.fumbbl.ffb.modifiers.bb2016.InterceptionModifierCollection();
		assertTrue(collection.getModifiers().stream().anyMatch(m -> "Pouring Rain".equals(m.getName())));
	}

	@Test
	void base_includes_disturbing_presence_modifier() {
		com.fumbbl.ffb.modifiers.InterceptionModifierCollection collection =
			new com.fumbbl.ffb.modifiers.bb2016.InterceptionModifierCollection();
		assertTrue(collection.getModifiers().stream().anyMatch(m -> "1 Disturbing Presence".equals(m.getName())));
	}

	@Test
	void base_all_modifiers_have_nonempty_names() {
		// Base content (11 disturbing_presence + pouring_rain) is a subset of this concrete subclass.
		Set<InterceptionModifier> modifiers = new com.fumbbl.ffb.modifiers.bb2016.InterceptionModifierCollection().getModifiers();
		assertTrue(modifiers.stream().allMatch(m -> m.getName() != null && !m.getName().isEmpty()));
	}

	// ── bb2016: base 12 + 8 tacklezone = 20 ──────────────────────────────────────────────────────

	@Test
	void bb2016_has_twenty_modifiers() {
		assertEquals(20, new com.fumbbl.ffb.modifiers.bb2016.InterceptionModifierCollection().getModifiers().size());
	}

	@Test
	void bb2016_includes_tacklezone_modifiers() {
		Set<InterceptionModifier> modifiers = new com.fumbbl.ffb.modifiers.bb2016.InterceptionModifierCollection().getModifiers();
		assertTrue(modifiers.stream().anyMatch(m -> "1 Tacklezone".equals(m.getName())));
		assertTrue(modifiers.stream().anyMatch(m -> "8 Tacklezones".equals(m.getName())));
	}

	@Test
	void bb2016_tacklezone_count_is_eight() {
		assertEquals(8, new com.fumbbl.ffb.modifiers.bb2016.InterceptionModifierCollection()
			.getModifiers(ModifierType.TACKLEZONE).size());
	}

	@Test
	void bb2016_includes_eight_tacklezones_modifier() {
		Set<InterceptionModifier> modifiers = new com.fumbbl.ffb.modifiers.bb2016.InterceptionModifierCollection().getModifiers();
		assertTrue(modifiers.stream().anyMatch(m -> "8 Tacklezones".equals(m.getName())));
	}

	@Test
	void bb2016_disturbing_presence_count_is_eleven() {
		assertEquals(11, new com.fumbbl.ffb.modifiers.bb2016.InterceptionModifierCollection()
			.getModifiers(ModifierType.DISTURBING_PRESENCE).size());
	}

	// ── bb2020: base 12 + 3 pass_result + 8 tacklezone + 1 stunty = 24 ───────────────────────────

	@Test
	void bb2020_has_twenty_four_modifiers() {
		assertEquals(24, new com.fumbbl.ffb.modifiers.bb2020.InterceptionModifierCollection().getModifiers().size());
	}

	@Test
	void bb2020_includes_accurate_pass_modifier() {
		Set<InterceptionModifier> modifiers = new com.fumbbl.ffb.modifiers.bb2020.InterceptionModifierCollection().getModifiers();
		assertTrue(modifiers.stream().anyMatch(m -> "Accurate Pass".equals(m.getName())));
	}

	@Test
	void bb2020_includes_stunty_modifier() {
		Set<InterceptionModifier> modifiers = new com.fumbbl.ffb.modifiers.bb2020.InterceptionModifierCollection().getModifiers();
		assertTrue(modifiers.stream().anyMatch(m -> "Thrower has Stunty".equals(m.getName())));
	}

	@Test
	void bb2020_includes_wildly_inaccurate_pass_modifier() {
		Set<InterceptionModifier> modifiers = new com.fumbbl.ffb.modifiers.bb2020.InterceptionModifierCollection().getModifiers();
		assertTrue(modifiers.stream().anyMatch(m -> "Wildly Inaccurate Pass".equals(m.getName())));
	}

	@Test
	void bb2020_tacklezone_count_is_eight() {
		assertEquals(8, new com.fumbbl.ffb.modifiers.bb2020.InterceptionModifierCollection()
			.getModifiers(ModifierType.TACKLEZONE).size());
	}

	// ── bb2025: base 12 + 2 pass_result + 8 tacklezone + 1 stunty = 23 ───────────────────────────

	@Test
	void bb2025_has_twenty_three_modifiers() {
		assertEquals(23, new com.fumbbl.ffb.modifiers.bb2025.InterceptionModifierCollection().getModifiers().size());
	}

	@Test
	void bb2025_includes_accurate_pass_modifier() {
		Set<InterceptionModifier> modifiers = new com.fumbbl.ffb.modifiers.bb2025.InterceptionModifierCollection().getModifiers();
		assertTrue(modifiers.stream().anyMatch(m -> "Accurate Pass".equals(m.getName())));
	}

	@Test
	void bb2025_tacklezone_count_is_eight() {
		assertEquals(8, new com.fumbbl.ffb.modifiers.bb2025.InterceptionModifierCollection()
			.getModifiers(ModifierType.TACKLEZONE).size());
	}

	@Test
	void bb2025_includes_inaccurate_pass_modifier() {
		Set<InterceptionModifier> modifiers = new com.fumbbl.ffb.modifiers.bb2025.InterceptionModifierCollection().getModifiers();
		assertTrue(modifiers.stream().anyMatch(m -> "Inaccurate Pass".equals(m.getName())));
	}

	@Test
	void bb2025_includes_stunty_modifier() {
		Set<InterceptionModifier> modifiers = new com.fumbbl.ffb.modifiers.bb2025.InterceptionModifierCollection().getModifiers();
		assertTrue(modifiers.stream().anyMatch(m -> "Thrower has Stunty".equals(m.getName())));
	}
}
