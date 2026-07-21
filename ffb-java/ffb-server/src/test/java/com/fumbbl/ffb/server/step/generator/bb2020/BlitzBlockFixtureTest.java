package com.fumbbl.ffb.server.step.generator.bb2020;

import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.server.GameState;
import com.fumbbl.ffb.server.fixture.GameFixture;
import com.fumbbl.ffb.server.fixture.GeneratorTestSupport;
import com.fumbbl.ffb.server.step.IStep;
import com.fumbbl.ffb.server.step.StepId;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirrors the Rust push-order tests in
 * {@code ffb-rust/crates/ffb-engine/src/step/generator/bb2020/blitz_block.rs}.
 */
public class BlitzBlockFixtureTest {

	private GameState gameState;

	@BeforeEach
	public void setUp() {
		gameState = GameFixture.createGameState(3, RulesCollection.Rules.BB2020);
	}

	private IStep[] build(boolean frenzyBlock) {
		new BlitzBlock().pushSequence(new BlitzBlock.SequenceParams(gameState, null, false, frenzyBlock, null));
		return GeneratorTestSupport.sequence(gameState);
	}

	// Rust: blitz_block_starts_with_init_blocking
	@Test
	public void blitzBlockStartsWithInitBlocking() {
		assertEquals(StepId.INIT_BLOCKING, build(false)[0].getId());
	}

	// Rust: blitz_block_ends_with_end_blocking
	@Test
	public void blitzBlockEndsWithEndBlocking() {
		IStep[] steps = build(false);
		assertEquals(StepId.END_BLOCKING, steps[steps.length - 1].getId());
	}

	// Rust: blitz_block_has_no_activation_block
	@Test
	public void blitzBlockHasNoActivationBlock() {
		assertFalse(GeneratorTestSupport.contains(build(false), StepId.INIT_ACTIVATION));
	}

	// Rust: blitz_block_frenzy_includes_foul_appearance_and_horns
	@Test
	public void blitzBlockFrenzyIncludesFoulAppearanceAndHorns() {
		IStep[] steps = build(true);
		assertTrue(GeneratorTestSupport.contains(steps, StepId.FOUL_APPEARANCE));
		assertTrue(GeneratorTestSupport.contains(steps, StepId.HORNS));
	}

	// Rust: blitz_block_no_frenzy_excludes_foul_appearance_but_has_horns
	@Test
	public void blitzBlockNoFrenzyExcludesFoulAppearanceButHasHorns() {
		IStep[] steps = build(false);
		assertFalse(GeneratorTestSupport.contains(steps, StepId.FOUL_APPEARANCE));
		assertTrue(GeneratorTestSupport.contains(steps, StepId.HORNS));
	}
}
