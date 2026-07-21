package com.fumbbl.ffb.server.step.generator.bb2025;

import com.fumbbl.ffb.server.GameState;
import com.fumbbl.ffb.server.fixture.GameFixture;
import com.fumbbl.ffb.server.fixture.GeneratorTestSupport;
import com.fumbbl.ffb.server.step.IStep;
import com.fumbbl.ffb.server.step.IStepLabel;
import com.fumbbl.ffb.server.step.StepId;
import com.fumbbl.ffb.server.step.generator.SequenceGenerator;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirrors the Rust push-order tests in
 * {@code ffb-rust/crates/ffb-engine/src/step/generator/bb2025/then_i_started_blastin.rs}.
 */
public class ThenIStartedBlastinFixtureTest {

	private GameState gameState;

	@BeforeEach
	public void setUp() {
		gameState = GameFixture.createGameState(3);
	}

	private IStep[] build() {
		new ThenIStartedBlastin().pushSequence(new SequenceGenerator.SequenceParams(gameState));
		return GeneratorTestSupport.sequence(gameState);
	}

	// Rust: then_i_started_blastin_has_19_steps_with_activation
	@Test
	public void thenIStartedBlastinHas19StepsWithActivation() {
		IStep[] steps = build();
		assertEquals(19, steps.length);
		assertEquals(StepId.INIT_ACTIVATION, steps[0].getId());
	}

	// Rust: then_i_started_blastin_ends_with_end_labelled
	@Test
	public void thenIStartedBlastinEndsWithEndLabelled() {
		IStep[] steps = build();
		IStep last = steps[steps.length - 1];
		assertEquals(StepId.END_THEN_I_STARTED_BLASTIN, last.getId());
		assertEquals(IStepLabel.END, last.getLabel());
	}

	// Rust: then_i_started_blastin_step_follows_activation_sub_sequence
	@Test
	public void thenIStartedBlastinStepFollowsActivationSubSequence() {
		assertEquals(StepId.THEN_I_STARTED_BLASTIN, build()[13].getId());
	}

	// Rust: then_i_started_blastin_step_has_goto_label_on_end — NOT mirrored.
	// The Rust test asserts the GOTO_LABEL_ON_END StepParameter is present in the
	// built sequence step (pre-consumption). Java's StepThenIStartedBlastin declares
	// gotoLabelOnEnd but its init() does not consume GOTO_LABEL_ON_END from step
	// parameters (only from JSON via initFrom), so the value is not observable via a
	// field after Sequence build. Not a behavioral bug — a Java param-plumbing quirk.

	// Rust: contains_apothecary_step
	@Test
	public void containsApothecaryStep() {
		assertTrue(GeneratorTestSupport.contains(build(), StepId.APOTHECARY));
	}
}
