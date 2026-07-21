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
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirrors the Rust push-order tests in
 * {@code ffb-rust/crates/ffb-engine/src/step/generator/bb2016/select.rs}.
 */
public class SelectFixtureTest {

	private GameState gameState;

	@BeforeEach
	public void setUp() {
		gameState = GameFixture.createGameState(3, RulesCollection.Rules.BB2016);
	}

	private IStep[] build(boolean updatePersistence) {
		new Select().pushSequence(
			new com.fumbbl.ffb.server.step.generator.Select.SequenceParams(gameState, updatePersistence));
		return GeneratorTestSupport.sequence(gameState);
	}

	// Rust: select_starts_with_init_selecting
	@Test
	public void selectStartsWithInitSelecting() {
		assertEquals(StepId.INIT_SELECTING, build(false)[0].getId());
	}

	// Rust: select_ends_with_end_selecting_labelled
	@Test
	public void selectEndsWithEndSelectingLabelled() {
		IStep[] steps = build(false);
		IStep last = steps[steps.length - 1];
		assertEquals(StepId.END_SELECTING, last.getId());
		assertEquals(IStepLabel.END_SELECTING, last.getLabel());
	}

	// Rust: select_has_bone_head_and_blood_lust
	@Test
	public void selectHasBoneHeadAndBloodLust() {
		IStep[] steps = build(false);
		assertTrue(GeneratorTestSupport.contains(steps, StepId.BONE_HEAD));
		assertTrue(GeneratorTestSupport.contains(steps, StepId.BLOOD_LUST));
	}

	// Rust: select_has_jump_up_and_stand_up
	@Test
	public void selectHasJumpUpAndStandUp() {
		IStep[] steps = build(false);
		assertTrue(GeneratorTestSupport.contains(steps, StepId.JUMP_UP));
		assertTrue(GeneratorTestSupport.contains(steps, StepId.STAND_UP));
	}

	// Rust: update_persistence_param_passed_to_init_selecting
	@Test
	public void updatePersistenceParamPassedToInitSelecting() {
		assertTrue(GeneratorTestSupport.booleanField(build(true)[0], "fUpdatePersistence"));
	}

	// Rust: select_has_nine_steps
	@Test
	public void selectHasNineSteps() {
		assertEquals(9, build(false).length);
	}
}
