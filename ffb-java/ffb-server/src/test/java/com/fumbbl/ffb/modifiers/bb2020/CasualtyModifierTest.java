package com.fumbbl.ffb.modifiers.bb2020;

import com.fumbbl.ffb.model.RosterPlayer;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirror of ffb-rust crates/ffb-mechanics/src/modifiers/bb2020/casualty_modifier.rs tests.
 *
 * NOTE: predicate_can_reject_player uses Rust's with_predicate builder (Rust-invented); the Java
 * CasualtyModifier has no predicate mechanism (appliesToContext is fixed) - that case is Rust-only.
 */
public class CasualtyModifierTest {

	// rust: get_modifier_returns_value
	@Test
	public void getModifierReturnsValue() {
		assertEquals(2, new CasualtyModifier("Test", 2).getModifier());
	}

	// rust: applies_to_context_true_when_no_predicate
	@Test
	public void appliesToContextTrueWhenNoPredicate() {
		assertTrue(new CasualtyModifier("Test", 1).appliesToContext(new RosterPlayer()));
	}

	// rust: report_string_includes_name_and_modifier
	@Test
	public void reportStringIncludesNameAndModifier() {
		String s = new CasualtyModifier("Mighty Blow", 1).reportString();
		assertTrue(s.contains("Mighty Blow"));
		assertTrue(s.contains("1"));
	}

	// rust: get_name_returns_stored_name
	@Test
	public void getNameReturnsStoredName() {
		assertEquals("Claws", new CasualtyModifier("Claws", 1).getName());
	}
}
