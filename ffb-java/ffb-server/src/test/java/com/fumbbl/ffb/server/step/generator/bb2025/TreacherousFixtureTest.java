package com.fumbbl.ffb.server.step.generator.bb2025;

import com.fumbbl.ffb.ApothecaryMode;
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

/**
 * Mirrors the Rust push-order tests in
 * {@code ffb-rust/crates/ffb-engine/src/step/generator/bb2025/treacherous.rs}.
 */
public class TreacherousFixtureTest {

	private GameState gameState;

	@BeforeEach
	public void setUp() {
		gameState = GameFixture.createGameState(3);
	}

	private IStep[] build(String failureLabel) {
		new Treacherous().pushSequence(new Treacherous.SequenceParams(gameState, failureLabel));
		return GeneratorTestSupport.sequence(gameState);
	}

	// Rust: treacherous_has_18_steps_with_activation
	@Test
	public void treacherousHas18StepsWithActivation() {
		IStep[] steps = build("X");
		assertEquals(18, steps.length);
		assertEquals(StepId.INIT_ACTIVATION, steps[0].getId());
	}

	// Rust: treacherous_is_labelled_end
	@Test
	public void treacherousIsLabelledEnd() {
		IStep t = GeneratorTestSupport.find(build("X"), StepId.TREACHEROUS);
		assertEquals(IStepLabel.END, t.getLabel());
	}

	// Rust: failure_label_wired_to_treacherous_step
	@Test
	public void failureLabelWiredToTreacherousStep() {
		IStep t = GeneratorTestSupport.find(build("MY_END"), StepId.TREACHEROUS);
		assertEquals("MY_END", GeneratorTestSupport.readField(t, "goToLabelOnFailure"));
	}

	// Rust: jump_up_follows_activation_sub_sequence
	@Test
	public void jumpUpFollowsActivationSubSequence() {
		assertEquals(StepId.JUMP_UP, build("X")[13].getId());
	}

	// Rust: contains_apothecary_step_with_defender_mode
	@Test
	public void containsApothecaryStepWithDefenderMode() {
		IStep[] steps = build("X");
		IStep lastApothecary = null;
		for (IStep step : steps) {
			if (step.getId() == StepId.APOTHECARY) {
				lastApothecary = step;
			}
		}
		assertNotNull(lastApothecary);
		assertEquals(ApothecaryMode.DEFENDER, GeneratorTestSupport.readField(lastApothecary, "fApothecaryMode"));
	}
}
