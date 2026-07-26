package com.fumbbl.ffb.modifiers;

import com.fumbbl.ffb.mechanics.StatsMechanic;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirror of ffb-rust crates/ffb-mechanics/src/modifiers/temporary_stat_decrementer.rs tests.
 * The Java ctor derives the limit from a StatsMechanic (Rust passes an explicit PlayerStatLimit);
 * the limit-bounds assertion uses the mixed StatsMechanic's real bounds (AV = (3,11), matches Rust).
 */
public class TemporaryStatDecrementerTest {

	private final StatsMechanic mechanic = new com.fumbbl.ffb.mechanics.mixed.StatsMechanic();

	private TemporaryStatDecrementer dec(PlayerStatKey key) {
		return new TemporaryStatDecrementer(key, mechanic);
	}

	// rust: decrementer_apply_subtracts_one
	@Test
	public void decrementerApplySubtractsOne() {
		TemporaryStatDecrementer m = dec(PlayerStatKey.MA);
		assertEquals(4, m.apply(5));
		assertEquals(0, m.apply(1));
	}

	// rust: decrementer_applies_to_correct_stat
	@Test
	public void decrementerAppliesToCorrectStat() {
		TemporaryStatDecrementer m = dec(PlayerStatKey.AG);
		assertTrue(m.appliesTo(PlayerStatKey.AG));
		assertFalse(m.appliesTo(PlayerStatKey.MA));
	}

	// rust: decrementer_does_not_apply_to_other_stats
	@Test
	public void decrementerDoesNotApplyToOtherStats() {
		TemporaryStatDecrementer m = dec(PlayerStatKey.ST);
		assertFalse(m.appliesTo(PlayerStatKey.AV));
		assertFalse(m.appliesTo(PlayerStatKey.PA));
		assertFalse(m.appliesTo(PlayerStatKey.MA));
	}

	// rust: decrementer_limit_matches
	@Test
	public void decrementerLimitMatches() {
		TemporaryStatDecrementer m = dec(PlayerStatKey.AV);
		assertEquals(3, m.getLimit().getMin());
		assertEquals(11, m.getLimit().getMax());
	}

	// rust: decrementer_apply_from_zero_goes_negative
	@Test
	public void decrementerApplyFromZeroGoesNegative() {
		TemporaryStatDecrementer m = dec(PlayerStatKey.MA);
		assertEquals(-1, m.apply(0));
	}
}
