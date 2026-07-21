package com.fumbbl.ffb.server.step.generator.bb2025;

import com.fumbbl.ffb.server.GameState;
import com.fumbbl.ffb.server.fixture.GameFixture;
import com.fumbbl.ffb.server.fixture.GeneratorTestSupport;
import com.fumbbl.ffb.server.step.IStep;
import com.fumbbl.ffb.server.step.IStepLabel;
import com.fumbbl.ffb.server.step.StepId;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNotNull;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirrors the Rust push-order tests in
 * {@code ffb-rust/crates/ffb-engine/src/step/generator/bb2025/block.rs}.
 */
public class BlockFixtureTest {

	private GameState gameState;

	@BeforeEach
	public void setUp() {
		gameState = GameFixture.createGameState(3);
	}

	private IStep[] buildDefault() {
		new Block().pushSequence(new com.fumbbl.ffb.server.step.generator.Block.Builder(gameState).build());
		return GeneratorTestSupport.sequence(gameState);
	}

	// Rust: block_sequence_starts_with_init_blocking
	@Test
	public void blockSequenceStartsWithInitBlocking() {
		assertEquals(StepId.INIT_BLOCKING, buildDefault()[0].getId());
	}

	// Rust: block_sequence_ends_with_end_blocking
	@Test
	public void blockSequenceEndsWithEndBlocking() {
		IStep[] steps = buildDefault();
		IStep last = steps[steps.length - 1];
		assertEquals(StepId.END_BLOCKING, last.getId());
		assertEquals(IStepLabel.END_BLOCKING, last.getLabel());
	}

	// Rust: block_sequence_contains_block_roll
	@Test
	public void blockSequenceContainsBlockRoll() {
		assertTrue(GeneratorTestSupport.contains(buildDefault(), StepId.BLOCK_ROLL));
	}

	// Rust: block_sequence_contains_pushback_labelled
	@Test
	public void blockSequenceContainsPushbackLabelled() {
		IStep pushback = GeneratorTestSupport.find(buildDefault(), StepId.PUSHBACK);
		assertNotNull(pushback);
		assertEquals(IStepLabel.PUSHBACK, pushback.getLabel());
	}

	// Rust: block_sequence_contains_drop_falling_players_labelled
	@Test
	public void blockSequenceContainsDropFallingPlayersLabelled() {
		IStep dfp = GeneratorTestSupport.find(buildDefault(), StepId.DROP_FALLING_PLAYERS);
		assertNotNull(dfp);
		assertEquals(IStepLabel.DROP_FALLING_PLAYERS, dfp.getLabel());
	}

	// Rust: block_sequence_defender_id_in_init_params
	@Test
	public void blockSequenceDefenderIdInInitParams() {
		new Block().pushSequence(
			new com.fumbbl.ffb.server.step.generator.Block.Builder(gameState).withDefenderId("p42").build());
		IStep init = GeneratorTestSupport.sequence(gameState)[0];
		assertEquals("p42", GeneratorTestSupport.readField(init, "fBlockDefenderId"));
	}
}
