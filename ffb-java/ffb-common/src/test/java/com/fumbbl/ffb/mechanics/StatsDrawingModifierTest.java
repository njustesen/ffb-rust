package com.fumbbl.ffb.mechanics;

import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirror of ffb-rust crates/ffb-engine/src/mechanics/stats_drawing_modifier.rs tests.
 * positiveImproves treats a positive modifier as an improvement (negative → impairment);
 * positiveImpairs is the reverse; zero is neutral. The absolute modifier is always |modifier|.
 */
public class StatsDrawingModifierTest {

	// rust: positive_improves_positive_is_improvement
	@Test
	public void positiveImprovesPositiveIsImprovement() {
		StatsDrawingModifier m = StatsDrawingModifier.positiveImproves(2);
		assertTrue(m.isImprovement());
		assertFalse(m.isImpairment());
		assertEquals(2, m.getAbsoluteModifier());
	}

	// rust: positive_improves_negative_is_impairment
	@Test
	public void positiveImprovesNegativeIsImpairment() {
		StatsDrawingModifier m = StatsDrawingModifier.positiveImproves(-1);
		assertFalse(m.isImprovement());
		assertTrue(m.isImpairment());
		assertEquals(1, m.getAbsoluteModifier());
	}

	// rust: positive_improves_zero_is_neutral
	@Test
	public void positiveImprovesZeroIsNeutral() {
		StatsDrawingModifier m = StatsDrawingModifier.positiveImproves(0);
		assertFalse(m.isImprovement());
		assertFalse(m.isImpairment());
		assertEquals(0, m.getAbsoluteModifier());
	}

	// rust: positive_impairs_positive_is_impairment
	@Test
	public void positiveImpairsPositiveIsImpairment() {
		StatsDrawingModifier m = StatsDrawingModifier.positiveImpairs(3);
		assertFalse(m.isImprovement());
		assertTrue(m.isImpairment());
		assertEquals(3, m.getAbsoluteModifier());
	}

	// rust: positive_impairs_negative_is_improvement
	@Test
	public void positiveImpairsNegativeIsImprovement() {
		StatsDrawingModifier m = StatsDrawingModifier.positiveImpairs(-2);
		assertTrue(m.isImprovement());
		assertFalse(m.isImpairment());
		assertEquals(2, m.getAbsoluteModifier());
	}
}
