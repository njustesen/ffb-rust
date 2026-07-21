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
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirrors the Rust push-order tests in
 * {@code ffb-rust/crates/ffb-engine/src/step/generator/bb2020/select.rs}.
 */
public class SelectFixtureTest {

	private GameState gameState;

	@BeforeEach
	public void setUp() {
		gameState = GameFixture.createGameState(3, RulesCollection.Rules.BB2020);
	}

	private IStep[] build() {
		new Select().pushSequence(
			new com.fumbbl.ffb.server.step.generator.Select.SequenceParams(gameState, false));
		return GeneratorTestSupport.sequence(gameState);
	}

	// Rust: select_starts_with_init_selecting
	@Test
	public void selectStartsWithInitSelecting() {
		assertEquals(StepId.INIT_SELECTING, build()[0].getId());
	}

	// Rust: select_ends_with_end_selecting
	@Test
	public void selectEndsWithEndSelecting() {
		IStep[] steps = build();
		assertEquals(StepId.END_SELECTING, steps[steps.length - 1].getId());
	}

	// Rust: select_has_activation_block
	@Test
	public void selectHasActivationBlock() {
		assertTrue(GeneratorTestSupport.contains(build(), StepId.INIT_ACTIVATION));
	}

	// Rust: select_reset_fumblerooskie_labelled_end_selecting
	@Test
	public void selectResetFumblerooskieLabelledEndSelecting() {
		assertEquals(IStepLabel.END_SELECTING,
			GeneratorTestSupport.find(build(), StepId.RESET_FUMBLEROOSKIE).getLabel());
	}

	// Rust: select_jump_up_is_labelled_next
	@Test
	public void selectJumpUpIsLabelledNext() {
		assertEquals(IStepLabel.NEXT, GeneratorTestSupport.find(build(), StepId.JUMP_UP).getLabel());
	}
}
