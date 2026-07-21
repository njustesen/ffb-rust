package com.fumbbl.ffb.server.step.generator.bb2016;

import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.server.GameState;
import com.fumbbl.ffb.server.fixture.GameFixture;
import com.fumbbl.ffb.server.fixture.GeneratorTestSupport;
import com.fumbbl.ffb.server.step.IStep;
import com.fumbbl.ffb.server.step.IStepLabel;
import com.fumbbl.ffb.server.step.StepId;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirrors the Rust push-order tests in
 * {@code ffb-rust/crates/ffb-engine/src/step/generator/bb2016/blitz_block.rs}.
 */
public class BlitzBlockFixtureTest {

	private GameState gameState;

	@BeforeEach
	public void setUp() {
		gameState = GameFixture.createGameState(3, RulesCollection.Rules.BB2016);
	}

	private IStep[] build(GameState gs, boolean frenzyBlock) {
		new BlitzBlock().pushSequence(new BlitzBlock.SequenceParams(gs, null, false, frenzyBlock, null));
		return GeneratorTestSupport.sequence(gs);
	}

	// Rust: blitz_block_sequence_starts_with_init_blocking
	@Test
	public void blitzBlockSequenceStartsWithInitBlocking() {
		assertEquals(StepId.INIT_BLOCKING, build(gameState, false)[0].getId());
	}

	// Rust: blitz_block_sequence_ends_with_end_blocking
	@Test
	public void blitzBlockSequenceEndsWithEndBlocking() {
		IStep[] steps = build(gameState, false);
		assertEquals(StepId.END_BLOCKING, steps[steps.length - 1].getId());
	}

	// Rust: blitz_block_omits_foul_appearance_when_frenzy
	@Test
	public void blitzBlockOmitsFoulAppearanceWhenFrenzy() {
		assertFalse(GeneratorTestSupport.contains(build(gameState, true), StepId.FOUL_APPEARANCE));
	}

	// Rust: blitz_block_has_apothecary_defender_label
	@Test
	public void blitzBlockHasApothecaryDefenderLabel() {
		assertTrue(GeneratorTestSupport.findLabelled(build(gameState, false),
			StepId.APOTHECARY, IStepLabel.APOTHECARY_DEFENDER) != null);
	}

	// Rust: blitz_block_delegates_to_block_sequence
	@Test
	public void blitzBlockDelegatesToBlockSequence() {
		int blitzLen = build(gameState, false).length;
		GameState blockGs = GameFixture.createGameState(3, RulesCollection.Rules.BB2016);
		new Block().pushSequence(new com.fumbbl.ffb.server.step.generator.Block.Builder(blockGs)
			.isFrenzyBlock(false).build());
		int blockLen = GeneratorTestSupport.sequence(blockGs).length;
		assertEquals(blockLen, blitzLen);
	}

	// Rust: blitz_block_has_block_roll
	@Test
	public void blitzBlockHasBlockRoll() {
		assertTrue(GeneratorTestSupport.contains(build(gameState, false), StepId.BLOCK_ROLL));
	}
}
