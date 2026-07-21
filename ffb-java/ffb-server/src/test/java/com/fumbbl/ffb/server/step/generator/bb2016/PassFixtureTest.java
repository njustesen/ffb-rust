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
 * {@code ffb-rust/crates/ffb-engine/src/step/generator/bb2016/pass.rs}.
 */
public class PassFixtureTest {

	private GameState gameState;

	@BeforeEach
	public void setUp() {
		gameState = GameFixture.createGameState(3, RulesCollection.Rules.BB2016);
	}

	private IStep[] build() {
		new Pass().pushSequence(new Pass.SequenceParams(gameState));
		return GeneratorTestSupport.sequence(gameState);
	}

	// Rust: pass_starts_with_init_passing
	@Test
	public void passStartsWithInitPassing() {
		assertEquals(StepId.INIT_PASSING, build()[0].getId());
	}

	// Rust: pass_ends_with_end_passing_labelled
	@Test
	public void passEndsWithEndPassingLabelled() {
		IStep[] steps = build();
		IStep last = steps[steps.length - 1];
		assertEquals(StepId.END_PASSING, last.getId());
		assertEquals(IStepLabel.END_PASSING, last.getLabel());
	}

	// Rust: pass_has_bone_head_and_blood_lust
	@Test
	public void passHasBoneHeadAndBloodLust() {
		IStep[] steps = build();
		assertTrue(GeneratorTestSupport.contains(steps, StepId.BONE_HEAD));
		assertTrue(GeneratorTestSupport.contains(steps, StepId.BLOOD_LUST));
	}

	// Rust: pass_pass_step_is_labelled
	@Test
	public void passPassStepIsLabelled() {
		assertEquals(IStepLabel.PASS, GeneratorTestSupport.findLabelled(build(), StepId.PASS, IStepLabel.PASS).getLabel());
	}

	// Rust: pass_catch_scatter_is_labelled_scatter_ball
	@Test
	public void passCatchScatterIsLabelledScatterBall() {
		assertNotNull(GeneratorTestSupport.findLabelled(build(),
			StepId.CATCH_SCATTER_THROW_IN, IStepLabel.SCATTER_BALL));
	}

	// Rust: pass_inserts_safe_throw_hook_after_intercept
	@Test
	public void passInsertsSafeThrowHookAfterIntercept() {
		IStep[] steps = build();
		int interceptIdx = GeneratorTestSupport.indexOf(steps, StepId.INTERCEPT);
		IStep safeThrow = steps[interceptIdx + 1];
		assertEquals(StepId.SAFE_THROW, safeThrow.getId());
		assertEquals(IStepLabel.END_PASSING, GeneratorTestSupport.readField(safeThrow, "fGotoLabelOnFailure"));
	}
}
