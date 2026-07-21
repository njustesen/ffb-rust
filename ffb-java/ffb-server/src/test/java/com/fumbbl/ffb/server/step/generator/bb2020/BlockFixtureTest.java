package com.fumbbl.ffb.server.step.generator.bb2020;

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
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirrors the Rust push-order tests in
 * {@code ffb-rust/crates/ffb-engine/src/step/generator/bb2020/block.rs}.
 */
public class BlockFixtureTest {

	private GameState gameState;

	@BeforeEach
	public void setUp() {
		gameState = GameFixture.createGameState(3, RulesCollection.Rules.BB2020);
	}

	private IStep[] build() {
		new Block().pushSequence(new com.fumbbl.ffb.server.step.generator.Block.Builder(gameState).build());
		return GeneratorTestSupport.sequence(gameState);
	}

	// Rust: block_sequence_starts_with_init_blocking
	@Test
	public void blockSequenceStartsWithInitBlocking() {
		assertEquals(StepId.INIT_BLOCKING, build()[0].getId());
	}

	// Rust: block_sequence_ends_with_end_blocking
	@Test
	public void blockSequenceEndsWithEndBlocking() {
		IStep[] steps = build();
		IStep last = steps[steps.length - 1];
		assertEquals(StepId.END_BLOCKING, last.getId());
		assertEquals(IStepLabel.END_BLOCKING, last.getLabel());
	}

	// Rust: block_has_activation_block
	@Test
	public void blockHasActivationBlock() {
		IStep[] steps = build();
		assertTrue(GeneratorTestSupport.contains(steps, StepId.INIT_ACTIVATION));
		assertTrue(GeneratorTestSupport.contains(steps, StepId.BONE_HEAD));
	}

	// Rust: block_pushback_is_labelled
	@Test
	public void blockPushbackIsLabelled() {
		assertEquals(IStepLabel.PUSHBACK, GeneratorTestSupport.find(build(), StepId.PUSHBACK).getLabel());
	}

	// Rust: block_drop_falling_players_is_labelled
	@Test
	public void blockDropFallingPlayersIsLabelled() {
		assertEquals(IStepLabel.DROP_FALLING_PLAYERS,
			GeneratorTestSupport.find(build(), StepId.DROP_FALLING_PLAYERS).getLabel());
	}

	// Rust: block_sequence_always_includes_set_defender_even_without_defender_id
	@Test
	public void blockSequenceAlwaysIncludesSetDefenderEvenWithoutDefenderId() {
		assertTrue(GeneratorTestSupport.contains(build(), StepId.SET_DEFENDER));
	}
}
