package com.fumbbl.ffb.server.step.generator.bb2025;

import com.fumbbl.ffb.server.GameState;
import com.fumbbl.ffb.server.fixture.GameFixture;
import com.fumbbl.ffb.server.fixture.GeneratorTestSupport;
import com.fumbbl.ffb.server.step.IStep;
import com.fumbbl.ffb.server.step.IStepLabel;
import com.fumbbl.ffb.server.step.StepId;
import com.fumbbl.ffb.server.step.generator.Sequence;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

/**
 * Mirrors the Rust tests in
 * {@code ffb-rust/crates/ffb-engine/src/step/generator/bb2025/activation_sequence_builder.rs}.
 */
public class ActivationSequenceBuilderFixtureTest {

	private GameState gameState;

	@BeforeEach
	public void setUp() {
		gameState = GameFixture.createGameState(3);
	}

	private IStep[] build(String failureLabel, String eventualDefender) {
		Sequence sequence = new Sequence(gameState);
		ActivationSequenceBuilder builder = ActivationSequenceBuilder.create().withFailureLabel(failureLabel);
		if (eventualDefender != null) {
			builder = builder.withEventualDefender(eventualDefender);
		}
		builder.addTo(sequence);
		gameState.getStepStack().push(sequence.getSequence());
		return GeneratorTestSupport.sequence(gameState);
	}

	// Rust: add_to_emits_thirteen_steps_without_defender
	@Test
	public void addToEmitsThirteenStepsWithoutDefender() {
		assertEquals(13, build("END", null).length);
	}

	// Rust: first_step_is_init_activation
	@Test
	public void firstStepIsInitActivation() {
		assertEquals(StepId.INIT_ACTIVATION, build("END", null)[0].getId());
	}

	// Rust: bone_head_has_next_label_at_index_8
	@Test
	public void boneHeadHasNextLabelAtIndex8() {
		IStep[] steps = build("END", null);
		assertEquals(StepId.BONE_HEAD, steps[8].getId());
		assertEquals(IStepLabel.NEXT, steps[8].getLabel());
	}

	// Rust: eventual_defender_adds_set_defender_and_emits_fourteen_steps
	@Test
	public void eventualDefenderAddsSetDefenderAndEmitsFourteenSteps() {
		IStep[] steps = build("END", "player1");
		assertEquals(14, steps.length);
		assertEquals(StepId.SET_DEFENDER, steps[7].getId());
	}
}
