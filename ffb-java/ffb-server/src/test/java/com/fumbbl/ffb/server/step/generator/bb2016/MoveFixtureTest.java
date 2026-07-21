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
import static org.junit.jupiter.api.Assertions.assertNotNull;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirrors the Rust push-order tests in
 * {@code ffb-rust/crates/ffb-engine/src/step/generator/bb2016/move_.rs}.
 */
public class MoveFixtureTest {

	private GameState gameState;

	@BeforeEach
	public void setUp() {
		gameState = GameFixture.createGameState(3, RulesCollection.Rules.BB2016);
	}

	private IStep[] build() {
		new Move().pushSequence(new Move.SequenceParams(gameState));
		return GeneratorTestSupport.sequence(gameState);
	}

	// Rust: move_starts_with_init_moving
	@Test
	public void moveStartsWithInitMoving() {
		assertEquals(StepId.INIT_MOVING, build()[0].getId());
	}

	// Rust: move_ends_with_end_moving
	@Test
	public void moveEndsWithEndMoving() {
		IStep[] steps = build();
		assertEquals(StepId.END_MOVING, steps[steps.length - 1].getId());
	}

	// Rust: move_contains_bone_head_and_blood_lust
	@Test
	public void moveContainsBoneHeadAndBloodLust() {
		IStep[] steps = build();
		assertTrue(GeneratorTestSupport.contains(steps, StepId.BONE_HEAD));
		assertTrue(GeneratorTestSupport.contains(steps, StepId.BLOOD_LUST));
	}

	// Rust: move_jump_has_move_start_param_when_set — NOT mirrored.
	// Java's bb2016 StepJump does not consume MOVE_START into a field (its StepState holds
	// only goToLabelOnFailure), so the param is not observable via the built step.

	// Rust: move_scatter_ball_labelled
	@Test
	public void moveScatterBallLabelled() {
		assertNotNull(GeneratorTestSupport.findLabelled(build(),
			StepId.CATCH_SCATTER_THROW_IN, IStepLabel.SCATTER_BALL));
	}
}
