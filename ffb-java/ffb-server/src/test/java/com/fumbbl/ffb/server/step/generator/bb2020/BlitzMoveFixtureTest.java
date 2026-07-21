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

/**
 * Mirrors the Rust push-order tests in
 * {@code ffb-rust/crates/ffb-engine/src/step/generator/bb2020/blitz_move.rs}.
 */
public class BlitzMoveFixtureTest {

	private GameState gameState;

	@BeforeEach
	public void setUp() {
		gameState = GameFixture.createGameState(3, RulesCollection.Rules.BB2020);
	}

	private IStep[] build() {
		new BlitzMove().pushSequence(new BlitzMove.SequenceParams(gameState));
		return GeneratorTestSupport.sequence(gameState);
	}

	// Rust: blitz_move_starts_with_init_moving
	@Test
	public void blitzMoveStartsWithInitMoving() {
		assertEquals(StepId.INIT_MOVING, build()[0].getId());
	}

	// Rust: blitz_move_ends_with_end_moving_labelled
	@Test
	public void blitzMoveEndsWithEndMovingLabelled() {
		IStep[] steps = build();
		IStep last = steps[steps.length - 1];
		assertEquals(StepId.END_MOVING, last.getId());
		assertEquals(IStepLabel.END_MOVING, last.getLabel());
	}

	// Rust: blitz_move_has_no_activation_block
	@Test
	public void blitzMoveHasNoActivationBlock() {
		IStep[] steps = build();
		assertFalse(GeneratorTestSupport.contains(steps, StepId.INIT_ACTIVATION));
		assertFalse(GeneratorTestSupport.contains(steps, StepId.BONE_HEAD));
	}

	// Rust: blitz_move_has_no_steady_footing
	@Test
	public void blitzMoveHasNoSteadyFooting() {
		assertFalse(GeneratorTestSupport.contains(build(), StepId.STEADY_FOOTING));
	}

	// Rust: blitz_move_trap_door_is_labelled_scatter_ball
	@Test
	public void blitzMoveTrapDoorIsLabelledScatterBall() {
		assertEquals(IStepLabel.SCATTER_BALL, GeneratorTestSupport.find(build(), StepId.TRAP_DOOR).getLabel());
	}
}
