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
import static org.junit.jupiter.api.Assertions.assertNotNull;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirrors the Rust push-order tests in
 * {@code ffb-rust/crates/ffb-engine/src/step/generator/bb2025/look_into_my_eyes.rs}.
 */
public class LookIntoMyEyesFixtureTest {

	private GameState gameState;

	@BeforeEach
	public void setUp() {
		gameState = GameFixture.createGameState(3);
	}

	private IStep[] build(boolean pushSelect, String gotoOnEnd) {
		new LookIntoMyEyes().pushSequence(new LookIntoMyEyes.SequenceParams(gameState, pushSelect, gotoOnEnd));
		return GeneratorTestSupport.sequence(gameState);
	}

	// Rust: look_into_my_eyes_has_16_steps_with_activation
	@Test
	public void lookIntoMyEyesHas16StepsWithActivation() {
		IStep[] steps = build(false, "");
		assertEquals(16, steps.length);
		assertEquals(StepId.INIT_ACTIVATION, steps[0].getId());
	}

	// Rust: look_into_my_eyes_step_is_labelled_end
	@Test
	public void lookIntoMyEyesStepIsLabelledEnd() {
		assertNotNull(GeneratorTestSupport.findLabelled(build(false, ""),
			StepId.LOOK_INTO_MY_EYES, IStepLabel.END));
	}

	// Rust: push_select_param_wired
	@Test
	public void pushSelectParamWired() {
		IStep step = GeneratorTestSupport.find(build(true, ""), StepId.LOOK_INTO_MY_EYES);
		assertTrue(GeneratorTestSupport.booleanField(step, "pushSelect"));
	}

	// Rust: goto_on_end_wired
	@Test
	public void gotoOnEndWired() {
		IStep step = GeneratorTestSupport.find(build(false, "MY_LABEL"), StepId.LOOK_INTO_MY_EYES);
		assertEquals("MY_LABEL", GeneratorTestSupport.readField(step, "gotoOnEnd"));
	}

	// Rust: init_look_into_my_eyes_follows_activation_sub_sequence
	@Test
	public void initLookIntoMyEyesFollowsActivationSubSequence() {
		assertEquals(StepId.INIT_LOOK_INTO_MY_EYES, build(false, "")[13].getId());
	}
}
