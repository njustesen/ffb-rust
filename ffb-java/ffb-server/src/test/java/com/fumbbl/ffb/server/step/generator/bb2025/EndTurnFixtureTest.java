package com.fumbbl.ffb.server.step.generator.bb2025;

import com.fumbbl.ffb.server.GameState;
import com.fumbbl.ffb.server.fixture.GameFixture;
import com.fumbbl.ffb.server.fixture.GeneratorTestSupport;
import com.fumbbl.ffb.server.step.IStep;
import com.fumbbl.ffb.server.step.StepId;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirrors the Rust push-order tests in
 * {@code ffb-rust/crates/ffb-engine/src/step/generator/bb2025/end_turn.rs}.
 */
public class EndTurnFixtureTest {

	private GameState gameState;

	@BeforeEach
	public void setUp() {
		gameState = GameFixture.createGameState(3);
	}

	private IStep[] build(boolean checkForgo) {
		new EndTurn().pushSequence(new EndTurn.SequenceParams(gameState, checkForgo));
		return GeneratorTestSupport.sequence(gameState);
	}

	// Rust: end_turn_sequence_has_six_steps
	@Test
	public void endTurnSequenceHasSixSteps() {
		assertEquals(6, build(false).length);
	}

	// Rust: end_turn_sequence_starts_with_forgone_stalling
	@Test
	public void endTurnSequenceStartsWithForgoneStalling() {
		assertEquals(StepId.FORGONE_STALLING, build(false)[0].getId());
	}

	// Rust: end_turn_sequence_ends_with_end_turn
	@Test
	public void endTurnSequenceEndsWithEndTurn() {
		IStep[] steps = build(false);
		assertEquals(StepId.END_TURN, steps[steps.length - 1].getId());
	}

	// Rust: end_turn_check_forgo_param_is_forwarded
	@Test
	public void endTurnCheckForgoParamIsForwarded() {
		IStep forgoneStalling = build(true)[0];
		assertTrue(GeneratorTestSupport.booleanField(forgoneStalling, "checkForgo"));
	}

	// Rust: build_sequence_returns_vec
	@Test
	public void buildSequenceReturnsVec() {
		assertTrue(build(false).length > 0);
	}
}
