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

/**
 * Mirrors the Rust push-order tests in
 * {@code ffb-rust/crates/ffb-engine/src/step/generator/bb2025/catch_of_the_day.rs}.
 */
public class CatchOfTheDayFixtureTest {

	private GameState gameState;

	@BeforeEach
	public void setUp() {
		gameState = GameFixture.createGameState(3);
	}

	private IStep[] build(String failureLabel) {
		new CatchOfTheDay().pushSequence(new CatchOfTheDay.SequenceParams(gameState, failureLabel));
		return GeneratorTestSupport.sequence(gameState);
	}

	// Rust: catch_of_the_day_last_step_labelled_end
	@Test
	public void catchOfTheDayLastStepLabelledEnd() {
		IStep[] steps = build("X");
		IStep last = steps[steps.length - 1];
		assertEquals(StepId.CATCH_OF_THE_DAY, last.getId());
		assertEquals(IStepLabel.END, last.getLabel());
	}

	// Rust: failure_label_in_params
	@Test
	public void failureLabelInParams() {
		IStep[] steps = build("lbl");
		assertEquals("lbl", GeneratorTestSupport.readField(steps[steps.length - 1], "goToLabelOnFailure"));
	}

	// Rust: activation_sub_sequence_precedes_catch_of_the_day
	@Test
	public void activationSubSequencePrecedesCatchOfTheDay() {
		IStep[] steps = build("X");
		assertEquals(14, steps.length);
		assertEquals(StepId.INIT_ACTIVATION, steps[0].getId());
	}
}
