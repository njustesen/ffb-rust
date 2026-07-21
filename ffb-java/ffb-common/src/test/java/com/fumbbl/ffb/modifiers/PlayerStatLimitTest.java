package com.fumbbl.ffb.modifiers;

import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

/**
 * 1:1 mirror of the Rust crates/ffb-mechanics/src/modifiers/player_stat_limit.rs unit tests.
 * Java class: {@link com.fumbbl.ffb.modifiers.PlayerStatLimit}.
 */
class PlayerStatLimitTest {

	@Test
	void new_stores_min_max() {
		PlayerStatLimit limit = new PlayerStatLimit(1, 9);
		assertEquals(1, limit.getMin());
		assertEquals(9, limit.getMax());
	}

	@Test
	void negative_min_allowed() {
		PlayerStatLimit limit = new PlayerStatLimit(-5, 5);
		assertEquals(-5, limit.getMin());
		assertEquals(5, limit.getMax());
	}

	// SKIPPED: Rust default_is_zero_zero — Java PlayerStatLimit has no default/no-arg constructor
	// (only PlayerStatLimit(min, max)); the Rust Default trait behaviour is inexpressible.

	// SKIPPED: Rust equality_by_value, inequality_on_different_bounds, copy_semantics — Java
	// PlayerStatLimit does not override equals()/hashCode(), so it uses identity equality rather
	// than the value/derived-PartialEq+Copy equality the Rust tests assert. Porting these with
	// assertEquals would test Object identity, not the value equality the Rust tests intend, so
	// they are inexpressible against the current Java class.
}
