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
import static org.junit.jupiter.api.Assertions.assertNull;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirrors the Rust push-order tests in
 * {@code ffb-rust/crates/ffb-engine/src/step/generator/bb2020/look_into_my_eyes.rs}.
 */
public class LookIntoMyEyesFixtureTest {

	private GameState gameState;

	@BeforeEach
	public void setUp() {
		gameState = GameFixture.createGameState(3, RulesCollection.Rules.BB2020);
	}

	private IStep[] build(boolean pushSelect, String gotoOnEnd) {
		new LookIntoMyEyes().pushSequence(new LookIntoMyEyes.SequenceParams(gameState, pushSelect, gotoOnEnd));
		return GeneratorTestSupport.sequence(gameState);
	}

	// Rust: look_into_my_eyes_has_activation_block
	@Test
	public void lookIntoMyEyesHasActivationBlock() {
		assertTrue(GeneratorTestSupport.contains(build(false, "end"), StepId.INIT_ACTIVATION));
	}

	// Rust: look_into_my_eyes_step_is_labelled_end
	@Test
	public void lookIntoMyEyesStepIsLabelledEnd() {
		assertEquals(IStepLabel.END, GeneratorTestSupport.find(build(false, "end"), StepId.LOOK_INTO_MY_EYES).getLabel());
	}

	// Rust: look_into_my_eyes_has_foul_appearance
	@Test
	public void lookIntoMyEyesHasFoulAppearance() {
		assertTrue(GeneratorTestSupport.contains(build(false, "end"), StepId.FOUL_APPEARANCE));
	}

	// Rust: look_into_my_eyes_blood_lust_has_no_failure_label
	@Test
	public void lookIntoMyEyesBloodLustHasNoFailureLabel() {
		IStep bloodLust = GeneratorTestSupport.find(build(false, "end"), StepId.BLOOD_LUST);
		Object state = GeneratorTestSupport.readField(bloodLust, "state");
		assertNull(GeneratorTestSupport.readField(state, "goToLabelOnFailure"));
	}

	// Rust: push_select_param_passed_to_look_into_my_eyes_step
	@Test
	public void pushSelectParamPassedToLookIntoMyEyesStep() {
		IStep s = GeneratorTestSupport.find(build(true, "end"), StepId.LOOK_INTO_MY_EYES);
		assertTrue(GeneratorTestSupport.booleanField(s, "pushSelect"));
	}

	// Rust: goto_on_end_param_passed_to_look_into_my_eyes_step
	@Test
	public void gotoOnEndParamPassedToLookIntoMyEyesStep() {
		IStep s = GeneratorTestSupport.find(build(false, "myEnd"), StepId.LOOK_INTO_MY_EYES);
		assertEquals("myEnd", GeneratorTestSupport.readField(s, "gotoOnEnd"));
	}
}
