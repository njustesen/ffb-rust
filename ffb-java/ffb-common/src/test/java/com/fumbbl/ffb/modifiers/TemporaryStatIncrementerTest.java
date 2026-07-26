package com.fumbbl.ffb.modifiers;

import com.fumbbl.ffb.mechanics.StatsMechanic;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirror of ffb-rust crates/ffb-mechanics/src/modifiers/temporary_stat_incrementer.rs tests.
 * The Java ctor derives the limit from a StatsMechanic (Rust passes an explicit PlayerStatLimit),
 * so the limit-bounds assertion uses the mixed StatsMechanic's real bounds for the stat.
 */
public class TemporaryStatIncrementerTest {

	private final StatsMechanic mechanic = new com.fumbbl.ffb.mechanics.mixed.StatsMechanic();

	private TemporaryStatIncrementer inc(PlayerStatKey key) {
		return new TemporaryStatIncrementer(key, mechanic);
	}

	// rust: incrementer_apply_adds_one
	@Test
	public void incrementerApplyAddsOne() {
		TemporaryStatIncrementer m = inc(PlayerStatKey.ST);
		assertEquals(4, m.apply(3));
		assertEquals(1, m.apply(0));
	}

	// rust: incrementer_applies_to_correct_stat
	@Test
	public void incrementerAppliesToCorrectStat() {
		TemporaryStatIncrementer m = inc(PlayerStatKey.AV);
		assertTrue(m.appliesTo(PlayerStatKey.AV));
		assertFalse(m.appliesTo(PlayerStatKey.ST));
	}

	// rust: incrementer_does_not_apply_to_other_stats
	@Test
	public void incrementerDoesNotApplyToOtherStats() {
		TemporaryStatIncrementer m = inc(PlayerStatKey.MA);
		assertFalse(m.appliesTo(PlayerStatKey.ST));
		assertFalse(m.appliesTo(PlayerStatKey.AG));
		assertFalse(m.appliesTo(PlayerStatKey.PA));
		assertFalse(m.appliesTo(PlayerStatKey.AV));
	}

	// rust: incrementer_limit_matches (Rust fabricates PA=(2,6); Java derives PA=(1,6) from mechanic)
	@Test
	public void incrementerLimitMatches() {
		TemporaryStatIncrementer m = inc(PlayerStatKey.PA);
		assertEquals(1, m.getLimit().getMin());
		assertEquals(6, m.getLimit().getMax());
	}

	// rust: incrementer_apply_from_max_still_adds_one (unconditional; clamping is caller's job)
	@Test
	public void incrementerApplyFromMaxStillAddsOne() {
		TemporaryStatIncrementer m = inc(PlayerStatKey.ST);
		assertEquals(10, m.apply(9));
	}
}
