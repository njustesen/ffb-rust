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
 * {@code ffb-rust/crates/ffb-engine/src/step/generator/bb2016/block.rs}.
 */
public class BlockFixtureTest {

	private GameState gameState;

	@BeforeEach
	public void setUp() {
		gameState = GameFixture.createGameState(3, RulesCollection.Rules.BB2016);
	}

	private IStep[] build(boolean frenzyBlock) {
		new Block().pushSequence(new com.fumbbl.ffb.server.step.generator.Block.Builder(gameState)
			.isFrenzyBlock(frenzyBlock).build());
		return GeneratorTestSupport.sequence(gameState);
	}

	// Rust: block_sequence_starts_with_init_blocking
	@Test
	public void blockSequenceStartsWithInitBlocking() {
		assertEquals(StepId.INIT_BLOCKING, build(false)[0].getId());
	}

	// Rust: block_sequence_ends_with_end_blocking
	@Test
	public void blockSequenceEndsWithEndBlocking() {
		IStep[] steps = build(false);
		IStep last = steps[steps.length - 1];
		assertEquals(StepId.END_BLOCKING, last.getId());
		assertEquals(IStepLabel.END_BLOCKING, last.getLabel());
	}

	// Rust: block_sequence_contains_bone_head
	@Test
	public void blockSequenceContainsBoneHead() {
		assertTrue(GeneratorTestSupport.contains(build(false), StepId.BONE_HEAD));
	}

	// Rust: block_sequence_includes_foul_appearance_when_not_frenzy
	@Test
	public void blockSequenceIncludesFoulAppearanceWhenNotFrenzy() {
		assertTrue(GeneratorTestSupport.contains(build(false), StepId.FOUL_APPEARANCE));
	}

	// Rust: block_sequence_omits_foul_appearance_when_frenzy
	@Test
	public void blockSequenceOmitsFoulAppearanceWhenFrenzy() {
		assertFalse(GeneratorTestSupport.contains(build(true), StepId.FOUL_APPEARANCE));
	}

	// Rust: block_sequence_has_apothecary_defender_label
	@Test
	public void blockSequenceHasApothecaryDefenderLabel() {
		assertTrue(GeneratorTestSupport.findLabelled(build(false),
			StepId.APOTHECARY, IStepLabel.APOTHECARY_DEFENDER) != null);
	}

	// Rust: block_sequence_pushback_is_labelled
	@Test
	public void blockSequencePushbackIsLabelled() {
		assertEquals(IStepLabel.PUSHBACK, GeneratorTestSupport.find(build(false), StepId.PUSHBACK).getLabel());
	}
}
