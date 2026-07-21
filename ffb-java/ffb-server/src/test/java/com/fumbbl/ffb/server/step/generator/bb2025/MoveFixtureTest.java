package com.fumbbl.ffb.server.step.generator.bb2025;

import com.fumbbl.ffb.server.GameState;
import com.fumbbl.ffb.server.fixture.GameFixture;
import com.fumbbl.ffb.server.step.IStep;
import com.fumbbl.ffb.server.step.IStepLabel;
import com.fumbbl.ffb.server.step.StepId;
import com.fumbbl.ffb.server.step.generator.GeneratorTestSupport;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNotNull;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirrors the Rust push-order tests in
 * {@code ffb-rust/crates/ffb-engine/src/step/generator/bb2025/move_.rs}.
 */
public class MoveFixtureTest {

	private GameState gameState;

	@BeforeEach
	public void setUp() {
		gameState = GameFixture.createGameState(3);
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

	// Rust: move_has_three_steady_footing_steps_labelled_steady_footing
	@Test
	public void moveHasThreeSteadyFootingStepsLabelledSteadyFooting() {
		IStep[] steps = build();
		int n = 0;
		for (IStep step : steps) {
			if (step.getId() == StepId.STEADY_FOOTING && IStepLabel.STEADY_FOOTING.equals(step.getLabel())) {
				n++;
			}
		}
		assertEquals(3, n);
	}

	// Rust: move_hypnotic_gaze_is_labelled_hypnotic_gaze
	@Test
	public void moveHypnoticGazeIsLabelledHypnoticGaze() {
		IStep hg = GeneratorTestSupport.find(build(), StepId.HYPNOTIC_GAZE);
		assertNotNull(hg);
		assertEquals(IStepLabel.HYPNOTIC_GAZE, hg.getLabel());
	}

	// Rust: move_retry_dodge_is_labelled
	@Test
	public void moveRetryDodgeIsLabelled() {
		assertNotNull(GeneratorTestSupport.findLabelled(build(), StepId.MOVE_DODGE, IStepLabel.RETRY_DODGE));
	}

	// Rust: move_drop_diving_tackler_fall_down_is_labelled_fall_down
	@Test
	public void moveDropDivingTacklerFallDownIsLabelledFallDown() {
		IStep[] steps = build();
		int n = 0;
		for (IStep step : steps) {
			if (step.getId() == StepId.DROP_DIVING_TACKLER && IStepLabel.FALL_DOWN.equals(step.getLabel())) {
				n++;
			}
		}
		assertEquals(1, n);
	}

	// Rust: move_shadowing_is_labelled_shadowing
	@Test
	public void moveShadowingIsLabelledShadowing() {
		assertNotNull(GeneratorTestSupport.findLabelled(build(), StepId.SHADOWING, IStepLabel.SHADOWING));
	}

	// Rust: move_trap_door_is_labelled_scatter_ball
	@Test
	public void moveTrapDoorIsLabelledScatterBall() {
		IStep td = GeneratorTestSupport.find(build(), StepId.TRAP_DOOR);
		assertNotNull(td);
		assertEquals(IStepLabel.SCATTER_BALL, td.getLabel());
	}

	// Rust: move_has_activation_block
	@Test
	public void moveHasActivationBlock() {
		IStep[] steps = build();
		assertTrue(GeneratorTestSupport.contains(steps, StepId.INIT_ACTIVATION));
		assertTrue(GeneratorTestSupport.contains(steps, StepId.BONE_HEAD));
	}

	// Rust: move_has_two_gfi_steps
	@Test
	public void moveHasTwoGfiSteps() {
		assertEquals(2, GeneratorTestSupport.count(build(), StepId.GO_FOR_IT));
	}
}
