package com.fumbbl.ffb.factory;

import com.fumbbl.ffb.modifiers.ArmorModifier;
import org.junit.jupiter.api.Test;

import java.util.Set;
import java.util.stream.Collectors;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirror of ffb-rust crates/ffb-mechanics/src/modifiers/{bb2016,bb2020,bb2025}/armor_modifiers.rs
 * tests — the per-edition ArmorModifiers collections (values/allValues/setUseAll membership).
 */
public class ArmorModifiersTest {

	private Set<String> names(java.util.stream.Stream<? extends ArmorModifier> stream) {
		return stream.map(ArmorModifier::getName).collect(Collectors.toSet());
	}

	// ── bb2016 ────────────────────────────────────────────────────────────────

	// rust: bb2016 values_has_correct_count
	@Test
	public void bb2016ValuesHasCorrectCount() {
		assertEquals(16, new com.fumbbl.ffb.factory.bb2016.ArmorModifiers().values().count());
	}

	// rust: bb2016 all_values_same_as_values
	@Test
	public void bb2016AllValuesSameAsValues() {
		com.fumbbl.ffb.factory.bb2016.ArmorModifiers m = new com.fumbbl.ffb.factory.bb2016.ArmorModifiers();
		assertEquals(m.values().count(), m.allValues().count());
	}

	// rust: bb2016 set_use_all_is_noop
	@Test
	public void bb2016SetUseAllIsNoop() {
		com.fumbbl.ffb.factory.bb2016.ArmorModifiers m = new com.fumbbl.ffb.factory.bb2016.ArmorModifiers();
		long before = m.values().count();
		m.setUseAll(true);
		assertEquals(before, m.values().count());
	}

	// rust: bb2016 contains_bomb_modifier
	@Test
	public void bb2016ContainsBombModifier() {
		assertTrue(names(new com.fumbbl.ffb.factory.bb2016.ArmorModifiers().values()).contains("Bomb"));
	}

	// rust: bb2016 get_name_is_nonempty
	@Test
	public void bb2016GetNameIsNonempty() {
		assertFalse(new com.fumbbl.ffb.factory.bb2016.ArmorModifiers().getName().isEmpty());
	}

	// ── bb2020 ────────────────────────────────────────────────────────────────

	// rust: bb2020 values_excludes_bomb_by_default
	@Test
	public void bb2020ValuesExcludesBombByDefault() {
		assertFalse(names(new com.fumbbl.ffb.factory.bb2020.ArmorModifiers().values()).contains("Bomb"));
	}

	// rust: bb2020 all_values_includes_bomb
	@Test
	public void bb2020AllValuesIncludesBomb() {
		assertTrue(names(new com.fumbbl.ffb.factory.bb2020.ArmorModifiers().allValues()).contains("Bomb"));
	}

	// rust: bb2020 set_use_all_includes_bomb_in_values
	@Test
	public void bb2020SetUseAllIncludesBombInValues() {
		com.fumbbl.ffb.factory.bb2020.ArmorModifiers m = new com.fumbbl.ffb.factory.bb2020.ArmorModifiers();
		m.setUseAll(true);
		assertTrue(names(m.values()).contains("Bomb"));
	}

	// rust: bb2020 base_count_is_fifteen
	@Test
	public void bb2020BaseCountIsFifteen() {
		assertEquals(15, new com.fumbbl.ffb.factory.bb2020.ArmorModifiers().values().count());
	}

	// rust: bb2020 get_name_is_nonempty
	@Test
	public void bb2020GetNameIsNonempty() {
		assertFalse(new com.fumbbl.ffb.factory.bb2020.ArmorModifiers().getName().isEmpty());
	}

	// ── bb2025 ────────────────────────────────────────────────────────────────

	// rust: bb2025 values_has_correct_count
	@Test
	public void bb2025ValuesHasCorrectCount() {
		assertEquals(15, new com.fumbbl.ffb.factory.bb2025.ArmorModifiers().values().count());
	}

	// rust: bb2025 all_values_same_as_values
	@Test
	public void bb2025AllValuesSameAsValues() {
		com.fumbbl.ffb.factory.bb2025.ArmorModifiers m = new com.fumbbl.ffb.factory.bb2025.ArmorModifiers();
		assertEquals(m.values().count(), m.allValues().count());
	}

	// rust: bb2025 no_bomb_modifier
	@Test
	public void bb2025NoBombModifier() {
		assertFalse(names(new com.fumbbl.ffb.factory.bb2025.ArmorModifiers().values()).contains("Bomb"));
	}

	// rust: bb2025 has_fireball_and_lightning
	@Test
	public void bb2025HasFireballAndLightning() {
		Set<String> names = names(new com.fumbbl.ffb.factory.bb2025.ArmorModifiers().values());
		assertTrue(names.contains("Fireball"));
		assertTrue(names.contains("Lightning"));
	}

	// rust: bb2025 get_name_is_nonempty
	@Test
	public void bb2025GetNameIsNonempty() {
		assertFalse(new com.fumbbl.ffb.factory.bb2025.ArmorModifiers().getName().isEmpty());
	}
}
