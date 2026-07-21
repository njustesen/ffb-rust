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
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertNotNull;

/**
 * Mirrors the Rust push-order tests in
 * {@code ffb-rust/crates/ffb-engine/src/step/generator/bb2025/blitz_move.rs}.
 */
public class BlitzMoveFixtureTest {

	private GameState gameState;

	@BeforeEach
	public void setUp() {
		gameState = GameFixture.createGameState(3);
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

	// Rust: blitz_move_has_no_foul_appearance_or_dump_off
	@Test
	public void blitzMoveHasNoFoulAppearanceOrDumpOff() {
		IStep[] steps = build();
		assertFalse(GeneratorTestSupport.contains(steps, StepId.FOUL_APPEARANCE));
		assertFalse(GeneratorTestSupport.contains(steps, StepId.DUMP_OFF));
		assertFalse(GeneratorTestSupport.contains(steps, StepId.HYPNOTIC_GAZE));
	}

	// Rust: blitz_move_has_three_steady_footing_steps_labelled
	@Test
	public void blitzMoveHasThreeSteadyFootingStepsLabelled() {
		IStep[] steps = build();
		int n = 0;
		for (IStep step : steps) {
			if (step.getId() == StepId.STEADY_FOOTING && IStepLabel.STEADY_FOOTING.equals(step.getLabel())) {
				n++;
			}
		}
		assertEquals(3, n);
	}

	// Rust: blitz_move_retry_dodge_is_labelled
	@Test
	public void blitzMoveRetryDodgeIsLabelled() {
		assertNotNull(GeneratorTestSupport.findLabelled(build(), StepId.MOVE_DODGE, IStepLabel.RETRY_DODGE));
	}

	// Rust: blitz_move_drop_diving_tackler_is_labelled_fall_down
	@Test
	public void blitzMoveDropDivingTacklerIsLabelledFallDown() {
		IStep[] steps = build();
		int n = 0;
		for (IStep step : steps) {
			if (step.getId() == StepId.DROP_DIVING_TACKLER && IStepLabel.FALL_DOWN.equals(step.getLabel())) {
				n++;
			}
		}
		assertEquals(1, n);
	}

	// Rust: blitz_move_trap_door_is_labelled_scatter_ball
	@Test
	public void blitzMoveTrapDoorIsLabelledScatterBall() {
		IStep td = GeneratorTestSupport.find(build(), StepId.TRAP_DOOR);
		assertNotNull(td);
		assertEquals(IStepLabel.SCATTER_BALL, td.getLabel());
	}

	// Rust: blitz_move_has_28_steps
	@Test
	public void blitzMoveHas28Steps() {
		assertEquals(28, build().length);
	}
}
