package com.fumbbl.ffb.modifiers;

import com.fumbbl.ffb.SpecialEffect;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertNull;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirror of ffb-rust crates/ffb-mechanics/src/modifiers/special_effect_injury_modifier.rs tests.
 * Java exposes the effect via getEffect() (Rust get_special_effect() trait override maps to it);
 * getModifier ignores its args (StaticInjuryModifier base).
 */
public class SpecialEffectInjuryModifierTest {

	// rust: stores_name_modifier_and_effect
	@Test
	public void storesNameModifierAndEffect() {
		SpecialEffectInjuryModifier m = new SpecialEffectInjuryModifier("Lightning Stun", 1, false, SpecialEffect.LIGHTNING);
		assertEquals("Lightning Stun", m.getName());
		assertEquals(1, m.getModifier(null, null));
		assertEquals(SpecialEffect.LIGHTNING, m.getEffect());
	}

	// rust: niggling_flag_propagates
	@Test
	public void nigglingFlagPropagates() {
		SpecialEffectInjuryModifier m = new SpecialEffectInjuryModifier("x", 0, true, SpecialEffect.BOMB);
		assertTrue(m.isNigglingInjuryModifier());
	}

	// rust: non_niggling_flag_is_false
	@Test
	public void nonNigglingFlagIsFalse() {
		SpecialEffectInjuryModifier m = new SpecialEffectInjuryModifier("Fireball", 1, false, SpecialEffect.FIREBALL);
		assertFalse(m.isNigglingInjuryModifier());
	}

	// rust: get_special_effect_returns_some (Java getEffect() returns the effect directly)
	@Test
	public void getSpecialEffectReturnsSome() {
		SpecialEffectInjuryModifier m = new SpecialEffectInjuryModifier("Bomb", 2, false, SpecialEffect.BOMB);
		assertEquals(SpecialEffect.BOMB, m.getEffect());
	}

	// rust: negative_modifier_stored_correctly
	@Test
	public void negativeModifierStoredCorrectly() {
		SpecialEffectInjuryModifier m = new SpecialEffectInjuryModifier("Heal", -1, false, SpecialEffect.LIGHTNING);
		assertEquals(-1, m.getModifier(null, null));
	}

	// rust: registered_to_default_is_none
	@Test
	public void registeredToDefaultIsNone() {
		SpecialEffectInjuryModifier m = new SpecialEffectInjuryModifier("X", 0, false, SpecialEffect.LIGHTNING);
		assertNull(m.getRegisteredTo());
	}
}
