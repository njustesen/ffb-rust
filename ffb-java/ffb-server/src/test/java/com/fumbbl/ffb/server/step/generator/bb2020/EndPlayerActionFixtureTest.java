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
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirrors the Rust push-order tests in
 * {@code ffb-rust/crates/ffb-engine/src/step/generator/bb2020/end_player_action.rs}.
 */
public class EndPlayerActionFixtureTest {

	private GameState gameState;

	@BeforeEach
	public void setUp() {
		gameState = GameFixture.createGameState(3, RulesCollection.Rules.BB2020);
	}

	private IStep[] build(boolean feedingAllowed) {
		new EndPlayerAction().pushSequence(
			new EndPlayerAction.SequenceParams(gameState, feedingAllowed, true, false, false));
		return GeneratorTestSupport.sequence(gameState);
	}

	// Rust: end_player_action_starts_with_remove_target_selection_state
	@Test
	public void endPlayerActionStartsWithRemoveTargetSelectionState() {
		assertEquals(StepId.REMOVE_TARGET_SELECTION_STATE, build(false)[0].getId());
	}

	// Rust: end_player_action_ends_with_end_feeding
	@Test
	public void endPlayerActionEndsWithEndFeeding() {
		IStep[] steps = build(false);
		assertEquals(StepId.END_FEEDING, steps[steps.length - 1].getId());
	}

	// Rust: check_stalling_is_labelled_end_feeding
	@Test
	public void checkStallingIsLabelledEndFeeding() {
		IStep cs = GeneratorTestSupport.find(build(false), StepId.CHECK_STALLING);
		assertEquals(IStepLabel.END_FEEDING, cs.getLabel());
	}

	// Rust: end_player_action_has_no_steady_footing
	@Test
	public void endPlayerActionHasNoSteadyFooting() {
		assertFalse(GeneratorTestSupport.contains(build(false), StepId.STEADY_FOOTING));
	}

	// Rust: feeding_allowed_param_flows_to_init_feeding
	@Test
	public void feedingAllowedParamFlowsToInitFeeding() {
		IStep initFeeding = GeneratorTestSupport.find(build(true), StepId.INIT_FEEDING);
		assertTrue(GeneratorTestSupport.booleanField(initFeeding, "fFeedingAllowed"));
	}

	// Rust: end_player_action_total_step_count
	@Test
	public void endPlayerActionTotalStepCount() {
		assertEquals(7, build(false).length);
	}
}
