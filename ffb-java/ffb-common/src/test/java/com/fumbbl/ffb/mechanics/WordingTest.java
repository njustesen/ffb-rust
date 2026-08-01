package com.fumbbl.ffb.mechanics;

import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

/**
 * Mirror of ffb-rust crates/ffb-engine/src/mechanics/wording.rs tests. Wording is an immutable
 * value holder for report phrasing (noun/verb/inflection/player characterization).
 *
 * The Rust equal_wordings_compare_equal / different_noun_is_not_equal / different_verb_is_not_equal /
 * clone_produces_equal_wording tests exercise Rust's DERIVED PartialEq/Eq/Clone; Java's Wording has
 * no equals()/hashCode()/clone() override (identity equality), so those cases have no Java counterpart
 * and are exempt.
 */
public class WordingTest {

	// rust: getters_return_constructed_values
	@Test
	public void gettersReturnConstructedValues() {
		Wording w = new Wording("Pass", "passes", "es", "thrower");
		assertEquals("Pass", w.getNoun());
		assertEquals("passes", w.getVerb());
		assertEquals("es", w.getInflection());
		assertEquals("thrower", w.getPlayerCharacterization());
	}

	// rust: empty_strings_accepted
	@Test
	public void emptyStringsAccepted() {
		Wording w = new Wording("", "", "", "");
		assertEquals("", w.getNoun());
		assertEquals("", w.getVerb());
		assertEquals("", w.getInflection());
		assertEquals("", w.getPlayerCharacterization());
	}
}
