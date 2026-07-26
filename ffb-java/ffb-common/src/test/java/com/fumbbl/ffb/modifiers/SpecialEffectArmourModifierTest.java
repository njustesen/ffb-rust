package com.fumbbl.ffb.modifiers;

import com.fumbbl.ffb.SpecialEffect;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertNull;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirror of ffb-rust crates/ffb-mechanics/src/modifiers/special_effect_armour_modifier.rs tests.
 * Java exposes the effect via getEffect() (the Rust get_special_effect() trait override maps to it);
 * getModifier ignores its args (StaticArmourModifier base).
 */
public class SpecialEffectArmourModifierTest {

	// rust: stores_name_modifier_and_effect
	@Test
	public void storesNameModifierAndEffect() {
		SpecialEffectArmourModifier m = new SpecialEffectArmourModifier("Lightning +3", 3, false, SpecialEffect.LIGHTNING);
		assertEquals("Lightning +3", m.getName());
		assertEquals(3, m.getModifier(null, null));
		assertEquals(SpecialEffect.LIGHTNING, m.getEffect());
	}

	// rust: effect_field_stored_correctly
	@Test
	public void effectFieldStoredCorrectly() {
		SpecialEffectArmourModifier m = new SpecialEffectArmourModifier("Fireball", 0, false, SpecialEffect.FIREBALL);
		assertEquals(SpecialEffect.FIREBALL, m.getEffect());
	}

	// rust: foul_assist_flag_propagates
	@Test
	public void foulAssistFlagPropagates() {
		SpecialEffectArmourModifier mNoFoul = new SpecialEffectArmourModifier("X", 0, false, SpecialEffect.LIGHTNING);
		assertFalse(mNoFoul.isFoulAssistModifier());
		SpecialEffectArmourModifier mFoul = new SpecialEffectArmourModifier("X", 0, true, SpecialEffect.LIGHTNING);
		assertTrue(mFoul.isFoulAssistModifier());
	}

	// rust: get_special_effect_returns_some (Java getEffect() returns the effect directly)
	@Test
	public void getSpecialEffectReturnsSome() {
		SpecialEffectArmourModifier m = new SpecialEffectArmourModifier("Bomb +2", 2, false, SpecialEffect.BOMB);
		assertEquals(SpecialEffect.BOMB, m.getEffect());
	}

	// rust: negative_modifier_stored_correctly
	@Test
	public void negativeModifierStoredCorrectly() {
		SpecialEffectArmourModifier m = new SpecialEffectArmourModifier("Reduce", -2, false, SpecialEffect.FIREBALL);
		assertEquals(-2, m.getModifier(null, null));
	}

	// rust: registered_to_default_is_none
	@Test
	public void registeredToDefaultIsNone() {
		SpecialEffectArmourModifier m = new SpecialEffectArmourModifier("X", 0, false, SpecialEffect.LIGHTNING);
		assertNull(m.getRegisteredTo());
	}
}
