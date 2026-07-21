package com.fumbbl.ffb.server.step.generator.bb2020;

import com.fumbbl.ffb.server.GameState;
import com.fumbbl.ffb.server.fixture.GameFixture;
import com.fumbbl.ffb.server.fixture.GeneratorTestSupport;
import com.fumbbl.ffb.server.step.IStep;
import com.fumbbl.ffb.server.step.IStepLabel;
import com.fumbbl.ffb.server.step.StepId;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNull;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirrors the Rust push-order tests in
 * {@code ffb-rust/crates/ffb-engine/src/step/generator/bb2020/catch_of_the_day.rs}.
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

	// Rust: catch_of_the_day_has_activation_block
	@Test
	public void catchOfTheDayHasActivationBlock() {
		assertTrue(GeneratorTestSupport.contains(build("X"), StepId.INIT_ACTIVATION));
	}

	// Rust: catch_of_the_day_ends_labelled_end
	@Test
	public void catchOfTheDayEndsLabelledEnd() {
		IStep[] steps = build("X");
		IStep last = steps[steps.length - 1];
		assertEquals(StepId.CATCH_OF_THE_DAY, last.getId());
		assertEquals(IStepLabel.END, last.getLabel());
	}

	// Rust: catch_of_the_day_bone_head_has_no_label
	@Test
	public void catchOfTheDayBoneHeadHasNoLabel() {
		assertNull(GeneratorTestSupport.find(build("X"), StepId.BONE_HEAD).getLabel());
	}

	// Rust: catch_of_the_day_blood_lust_has_no_failure_label — NOT mirrored
	// (StepBloodLust stores the label in a nested state field; param-absence not observable).

	// Rust: failure_label_passed_to_catch_of_the_day_step
	@Test
	public void failureLabelPassedToCatchOfTheDayStep() {
		IStep cotd = GeneratorTestSupport.find(build("myLabel"), StepId.CATCH_OF_THE_DAY);
		assertEquals("myLabel", GeneratorTestSupport.readField(cotd, "goToLabelOnFailure"));
	}

	// Rust: catch_of_the_day_step_count
	@Test
	public void catchOfTheDayStepCount() {
		assertEquals(12, build("X").length);
	}
}
