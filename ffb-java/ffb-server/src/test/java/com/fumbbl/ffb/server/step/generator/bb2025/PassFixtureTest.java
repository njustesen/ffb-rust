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

/**
 * Mirrors the Rust push-order tests in
 * {@code ffb-rust/crates/ffb-engine/src/step/generator/bb2025/pass.rs}.
 */
public class PassFixtureTest {

	private GameState gameState;

	@BeforeEach
	public void setUp() {
		gameState = GameFixture.createGameState(3);
	}

	private IStep[] build() {
		new Pass().pushSequence(new Pass.SequenceParams(gameState));
		return GeneratorTestSupport.sequence(gameState);
	}

	// Rust: pass_sequence_starts_with_init_passing
	@Test
	public void passSequenceStartsWithInitPassing() {
		assertEquals(StepId.INIT_PASSING, build()[0].getId());
	}

	// Rust: pass_sequence_ends_with_end_passing
	@Test
	public void passSequenceEndsWithEndPassing() {
		IStep[] steps = build();
		assertEquals(StepId.END_PASSING, steps[steps.length - 1].getId());
	}

	// Rust: pass_intercept_step_is_labelled
	@Test
	public void passInterceptStepIsLabelled() {
		IStep intercept = GeneratorTestSupport.find(build(), StepId.INTERCEPT);
		assertNotNull(intercept);
		assertEquals(IStepLabel.INTERCEPT, intercept.getLabel());
	}

	// Rust: pass_catch_scatter_is_labelled_scatter_ball
	@Test
	public void passCatchScatterIsLabelledScatterBall() {
		assertNotNull(GeneratorTestSupport.findLabelled(build(),
			StepId.CATCH_SCATTER_THROW_IN, IStepLabel.SCATTER_BALL));
	}

	// Rust: pass_inserts_no_intercept_hook_for_bb2025
	@Test
	public void passInsertsNoInterceptHookForBb2025() {
		IStep[] steps = build();
		int interceptIdx = GeneratorTestSupport.indexOf(steps, StepId.INTERCEPT);
		assertEquals(StepId.RESOLVE_PASS, steps[interceptIdx + 1].getId());
	}

	// Rust: pass_reset_to_move_is_labelled_end_passing
	@Test
	public void passResetToMoveIsLabelledEndPassing() {
		IStep rtm = GeneratorTestSupport.find(build(), StepId.RESET_TO_MOVE);
		assertNotNull(rtm);
		assertEquals(IStepLabel.END_PASSING, rtm.getLabel());
	}
}
