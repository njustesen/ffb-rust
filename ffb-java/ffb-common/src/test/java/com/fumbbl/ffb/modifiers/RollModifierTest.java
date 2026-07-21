package com.fumbbl.ffb.modifiers;

import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

/**
 * Mirror of the Rust crates/ffb-mechanics/src/modifiers/roll_modifier.rs unit tests.
 *
 * The Rust RollModifier is a concrete struct with its own name field, new()/with_report()
 * constructors and a Default impl. The Java {@link com.fumbbl.ffb.modifiers.RollModifier} is an
 * abstract base (RollModifier&lt;C&gt;) with no name field, no constructor and no default; its only
 * base-defined, subclass-independent behaviour is getMultiplier() defaulting to getModifier().
 * That behaviour is exercised here through a concrete subclass ({@link RightStuffModifier}) that
 * does not override getMultiplier(). The remaining Rust tests target the Rust struct's own
 * constructors/fields and are inexpressible against the abstract Java base (see skip notes below).
 */
class RollModifierTest {

	@Test
	void get_multiplier_equals_modifier() {
		// Java RollModifier.getMultiplier() defaults to getModifier(); RightStuffModifier does not
		// override it, so it exercises the abstract base behaviour asserted by the Rust test.
		RollModifier<?> m = new RightStuffModifier("x", 3, ModifierType.REGULAR);
		assertEquals(3, m.getMultiplier());
	}

	// SKIPPED: Rust new_stores_name_and_modifier — targets the Rust concrete RollModifier struct's
	// new(name, modifier) constructor and public `name` field. Java RollModifier is abstract with no
	// constructor and no name field, so this is inexpressible.

	// SKIPPED: Rust with_report_sets_all_fields — targets the Rust struct's with_report(name, report,
	// modifier, modifierIncluded) constructor. Java RollModifier is abstract with no such constructor;
	// report string / modifier-included are subclass responsibilities, so this is inexpressible.

	// SKIPPED: Rust default_has_empty_name_and_zero_modifier — targets the Rust struct's Default impl
	// and `name` field. Java RollModifier is abstract with no default constructor and no name field, so
	// this is inexpressible.
}
