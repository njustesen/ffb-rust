package com.fumbbl.ffb.modifiers;

import org.junit.jupiter.api.Test;

import java.util.Set;
import java.util.stream.Collectors;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirror of the Rust crates/ffb-mechanics/src/modifiers/gaze_modifier_collection.rs (abstract
 * base), modifiers/bb2016/gaze_modifier_collection.rs and modifiers/bb2020/gaze_modifier_collection.rs
 * unit tests.
 *
 * <p>The Java base {@link com.fumbbl.ffb.modifiers.GazeModifierCollection} is an empty abstract
 * class (no modifiers, {@code add} is protected). The Rust base module's tests all exercise the
 * Rust struct's public {@code new}/{@code add}/{@code default}/{@code find_applicable} surface,
 * which has no Java analog on the collection, so they are skipped (see the final report). Only the
 * bb2016 and bb2020 concrete subclasses have collection-level analogs.</p>
 */
class GazeModifierCollectionTest {

	// ── bb2016: 8 tacklezone modifiers, modifier = multiplier - 1 ─────────────────

	@Test
	void bb2016_creates_eight_modifiers() {
		assertEquals(8, new com.fumbbl.ffb.modifiers.bb2016.GazeModifierCollection().getModifiers().size());
	}

	@Test
	void bb2016_modifier_values_match_tacklezone_minus_one() {
		Set<GazeModifier> modifiers = new com.fumbbl.ffb.modifiers.bb2016.GazeModifierCollection().getModifiers();
		// each modifier's penalty is one less than its tacklezone-count multiplier
		assertTrue(modifiers.stream().allMatch(m -> m.getModifier() == m.getMultiplier() - 1));
		// multipliers span 1..=8
		Set<Integer> multipliers = modifiers.stream().map(GazeModifier::getMultiplier).collect(Collectors.toSet());
		assertEquals(8, multipliers.size());
		for (int i = 1; i <= 8; i++) {
			assertTrue(multipliers.contains(i), "expected a modifier with multiplier " + i);
		}
	}

	// ── bb2020: 8 tacklezone modifiers ────────────────────────────────────────────

	@Test
	void bb2020_has_eight_modifiers() {
		assertEquals(8, new com.fumbbl.ffb.modifiers.bb2020.GazeModifierCollection().getModifiers().size());
	}

	@Test
	void bb2020_includes_single_tacklezone_modifier() {
		Set<GazeModifier> modifiers = new com.fumbbl.ffb.modifiers.bb2020.GazeModifierCollection().getModifiers();
		assertTrue(modifiers.stream().anyMatch(m -> "1 Tacklezone".equals(m.getName())));
	}

	@Test
	void bb2020_all_modifiers_are_tacklezone_type() {
		Set<GazeModifier> modifiers = new com.fumbbl.ffb.modifiers.bb2020.GazeModifierCollection().getModifiers();
		assertTrue(modifiers.stream().allMatch(m -> m.getType() == ModifierType.TACKLEZONE));
	}

	@Test
	void bb2020_includes_eight_tacklezones_modifier() {
		Set<GazeModifier> modifiers = new com.fumbbl.ffb.modifiers.bb2020.GazeModifierCollection().getModifiers();
		assertTrue(modifiers.stream().anyMatch(m -> "8 Tacklezones".equals(m.getName())));
	}

	@Test
	void bb2020_tacklezone_count_is_eight() {
		assertEquals(8, new com.fumbbl.ffb.modifiers.bb2020.GazeModifierCollection()
			.getModifiers(ModifierType.TACKLEZONE).size());
	}
}
