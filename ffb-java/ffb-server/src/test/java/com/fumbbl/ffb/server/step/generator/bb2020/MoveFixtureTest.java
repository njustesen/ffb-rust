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
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirrors the Rust push-order tests in
 * {@code ffb-rust/crates/ffb-engine/src/step/generator/bb2020/move_.rs}.
 */
public class MoveFixtureTest {

	private GameState gameState;

	@BeforeEach
	public void setUp() {
		gameState = GameFixture.createGameState(3, RulesCollection.Rules.BB2020);
	}

	private IStep[] build() {
		new Move().pushSequence(new Move.SequenceParams(gameState));
		return GeneratorTestSupport.sequence(gameState);
	}

	// Rust: move_sequence_starts_with_init_moving
	@Test
	public void moveSequenceStartsWithInitMoving() {
		assertEquals(StepId.INIT_MOVING, build()[0].getId());
	}

	// Rust: move_sequence_ends_with_end_moving_labelled
	@Test
	public void moveSequenceEndsWithEndMovingLabelled() {
		IStep[] steps = build();
		IStep last = steps[steps.length - 1];
		assertEquals(StepId.END_MOVING, last.getId());
		assertEquals(IStepLabel.END_MOVING, last.getLabel());
	}

	// Rust: move_has_activation_block
	@Test
	public void moveHasActivationBlock() {
		IStep[] steps = build();
		assertTrue(GeneratorTestSupport.contains(steps, StepId.INIT_ACTIVATION));
		assertTrue(GeneratorTestSupport.contains(steps, StepId.BONE_HEAD));
	}

	// Rust: move_has_no_steady_footing
	@Test
	public void moveHasNoSteadyFooting() {
		assertFalse(GeneratorTestSupport.contains(build(), StepId.STEADY_FOOTING));
	}

	// Rust: move_has_two_gfi_steps
	@Test
	public void moveHasTwoGfiSteps() {
		assertEquals(2, GeneratorTestSupport.count(build(), StepId.GO_FOR_IT));
	}
}
