package com.fumbbl.ffb.server.step.generator.bb2016;

import com.fumbbl.ffb.ApothecaryMode;
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
 * {@code ffb-rust/crates/ffb-engine/src/step/generator/bb2016/end_player_action.rs}.
 */
public class EndPlayerActionFixtureTest {

	private GameState gameState;

	@BeforeEach
	public void setUp() {
		gameState = GameFixture.createGameState(3, RulesCollection.Rules.BB2016);
	}

	private IStep[] build(boolean feedingAllowed) {
		new EndPlayerAction().pushSequence(
			new EndPlayerAction.SequenceParams(gameState, feedingAllowed, true, false));
		return GeneratorTestSupport.sequence(gameState);
	}

	// Rust: end_player_action_starts_with_init_feeding
	@Test
	public void endPlayerActionStartsWithInitFeeding() {
		assertEquals(StepId.INIT_FEEDING, build(false)[0].getId());
	}

	// Rust: end_player_action_ends_with_end_feeding_labelled
	@Test
	public void endPlayerActionEndsWithEndFeedingLabelled() {
		IStep[] steps = build(false);
		IStep last = steps[steps.length - 1];
		assertEquals(StepId.END_FEEDING, last.getId());
		assertEquals(IStepLabel.END_FEEDING, last.getLabel());
	}

	// Rust: end_player_action_has_4_steps
	@Test
	public void endPlayerActionHas4Steps() {
		assertEquals(4, build(false).length);
	}

	// Rust: end_player_action_apothecary_has_feeding_mode
	@Test
	public void endPlayerActionApothecaryHasFeedingMode() {
		IStep apo = GeneratorTestSupport.find(build(false), StepId.APOTHECARY);
		assertEquals(ApothecaryMode.FEEDING, GeneratorTestSupport.readField(apo, "fApothecaryMode"));
	}

	// Rust: feeding_allowed_param_flows_to_init_feeding
	@Test
	public void feedingAllowedParamFlowsToInitFeeding() {
		IStep init = GeneratorTestSupport.find(build(true), StepId.INIT_FEEDING);
		assertTrue(GeneratorTestSupport.booleanField(init, "fFeedingAllowed"));
	}
}
