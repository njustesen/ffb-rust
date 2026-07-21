package com.fumbbl.ffb.modifiers;

import org.junit.jupiter.api.Test;

import java.util.Set;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirror of the Rust crates/ffb-mechanics/src/modifiers/pickup_modifier_collection.rs unit tests.
 * The Java {@link com.fumbbl.ffb.modifiers.PickupModifierCollection} is concrete (not abstract), seeded in
 * its constructor with a Pouring Rain modifier + 8 tacklezone modifiers, and lives in this test's package so
 * the protected add() is accessible. The default()==new() test is Rust-only and skipped.
 */
class PickupModifierCollectionTest {

	@Test
	void new_initializes_with_base_modifiers() {
		assertFalse(new PickupModifierCollection().getModifiers().isEmpty());
	}

	@Test
	void get_modifiers_by_type_tacklezone_returns_8_entries() {
		assertEquals(8, new PickupModifierCollection().getModifiers(ModifierType.TACKLEZONE).size());
	}

	@Test
	void add_increases_modifier_count() {
		PickupModifierCollection collection = new PickupModifierCollection();
		int countBefore = collection.getModifiers().size();
		collection.add(new PickupModifier("extra", 1, ModifierType.REGULAR));
		assertEquals(countBefore + 1, collection.getModifiers().size());
	}

	@Test
	void all_modifiers_have_nonempty_names() {
		Set<PickupModifier> modifiers = new PickupModifierCollection().getModifiers();
		assertTrue(modifiers.stream().allMatch(m -> m.getName() != null && !m.getName().isEmpty()));
	}
}
