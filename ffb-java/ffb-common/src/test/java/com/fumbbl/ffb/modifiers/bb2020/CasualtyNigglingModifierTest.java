package com.fumbbl.ffb.modifiers.bb2020;

import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

/**
 * Mirror of ffb-rust crates/ffb-mechanics/src/modifiers/bb2020/casualty_niggling_modifier.rs tests.
 * Extends CasualtyModifier (getName/getModifier inherited); reportString() == getName().
 */
public class CasualtyNigglingModifierTest {

	// rust: get_modifier_returns_value
	@Test
	public void getModifierReturnsValue() {
		CasualtyNigglingModifier m = new CasualtyNigglingModifier("Niggling Injury", 1);
		assertEquals(1, m.getModifier());
	}

	// rust: report_string_returns_name
	@Test
	public void reportStringReturnsName() {
		CasualtyNigglingModifier m = new CasualtyNigglingModifier("Niggling Injury", 1);
		assertEquals("Niggling Injury", m.reportString());
	}

	// rust: get_name_returns_name
	@Test
	public void getNameReturnsName() {
		CasualtyNigglingModifier m = new CasualtyNigglingModifier("Double Niggling", 2);
		assertEquals("Double Niggling", m.getName());
	}

	// rust: get_modifier_returns_various_values
	@Test
	public void getModifierReturnsVariousValues() {
		assertEquals(0, new CasualtyNigglingModifier("x", 0).getModifier());
		assertEquals(3, new CasualtyNigglingModifier("x", 3).getModifier());
		assertEquals(-1, new CasualtyNigglingModifier("x", -1).getModifier());
	}

	// rust: report_string_matches_get_name
	@Test
	public void reportStringMatchesGetName() {
		CasualtyNigglingModifier m = new CasualtyNigglingModifier("Three Nigglings", 3);
		assertEquals(m.getName(), m.reportString());
	}
}
