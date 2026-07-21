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
 * {@code ffb-rust/crates/ffb-engine/src/step/generator/bb2025/end_player_action.rs}.
 */
public class EndPlayerActionFixtureTest {

	private GameState gameState;

	@BeforeEach
	public void setUp() {
		gameState = GameFixture.createGameState(3);
	}

	private IStep[] build(boolean feedingAllowed, boolean checkForgo) {
		new EndPlayerAction().pushSequence(
			new EndPlayerAction.SequenceParams(gameState, feedingAllowed, true, false, checkForgo));
		return GeneratorTestSupport.sequence(gameState);
	}

	// Rust: end_player_action_starts_with_remove_target_selection_state
	@Test
	public void endPlayerActionStartsWithRemoveTargetSelectionState() {
		assertEquals(StepId.REMOVE_TARGET_SELECTION_STATE, build(false, false)[0].getId());
	}

	// Rust: end_player_action_ends_with_end_feeding
	@Test
	public void endPlayerActionEndsWithEndFeeding() {
		IStep[] steps = build(false, false);
		assertEquals(StepId.END_FEEDING, steps[steps.length - 1].getId());
	}

	// Rust: stalling_player_is_labelled_end_feeding
	@Test
	public void stallingPlayerIsLabelledEndFeeding() {
		assertNotNull(GeneratorTestSupport.findLabelled(build(false, false),
			StepId.STALLING_PLAYER, IStepLabel.END_FEEDING));
	}

	// Rust: total_step_count_is_eleven
	@Test
	public void totalStepCountIsEleven() {
		assertEquals(11, build(false, false).length);
	}

	// Rust: check_forgo_param_flows_to_end_feeding
	@Test
	public void checkForgoParamFlowsToEndFeeding() {
		IStep endFeeding = GeneratorTestSupport.find(build(false, true), StepId.END_FEEDING);
		assertTrue(GeneratorTestSupport.booleanField(endFeeding, "checkForgo"));
	}

	// Rust: feeding_allowed_param_flows_to_init_feeding
	@Test
	public void feedingAllowedParamFlowsToInitFeeding() {
		IStep initFeeding = GeneratorTestSupport.find(build(true, false), StepId.INIT_FEEDING);
		assertTrue(GeneratorTestSupport.booleanField(initFeeding, "fFeedingAllowed"));
	}
}
