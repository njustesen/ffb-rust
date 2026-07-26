package com.fumbbl.ffb.server.step.bb2016.foul;

import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.server.GameState;
import com.fumbbl.ffb.server.fixture.GameFixture;
import com.fumbbl.ffb.server.fixture.GeneratorTestSupport;
import com.fumbbl.ffb.server.step.IStep;
import com.fumbbl.ffb.server.step.StepAction;
import com.fumbbl.ffb.server.step.StepId;
import com.fumbbl.ffb.server.step.StepParameter;
import com.fumbbl.ffb.server.step.StepParameterKey;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirrors the Rust tests in
 * {@code ffb-rust/crates/ffb-engine/src/step/bb2016/foul/step_end_fouling.rs}.
 * StepId.END_FOULING always pushes the EndPlayerAction sequence (first step INIT_FEEDING), passing
 * the END_TURN flag through to it.
 */
public class StepEndFoulingFixtureTest {

	private GameState gameState;

	@BeforeEach
	public void setUp() {
		gameState = GameFixture.createGameState(3, RulesCollection.Rules.BB2016);
	}

	private IStep newStep() {
		return GameFixture.createStep(gameState, StepId.END_FOULING);
	}

	// rust: pushes_end_player_action_sequence
	@Test
	public void pushesEndPlayerActionSequence() {
		IStep step = newStep();
		assertEquals(StepAction.NEXT_STEP, GameFixture.startStep(step));
		IStep[] seq = GeneratorTestSupport.sequence(gameState);
		assertTrue(seq.length > 0);
		assertEquals(StepId.INIT_FEEDING, seq[0].getId());
	}

	// rust: end_turn_passes_through_to_sequence
	@Test
	public void endTurnPassesThroughToSequence() {
		IStep step = newStep();
		step.setParameter(StepParameter.from(StepParameterKey.END_TURN, true));
		GameFixture.startStep(step);
		IStep[] seq = GeneratorTestSupport.sequence(gameState);
		assertEquals(StepId.INIT_FEEDING, seq[0].getId());
		assertTrue(GeneratorTestSupport.booleanField(seq[0], "fEndTurn"));
	}

	// rust: set_parameter_end_turn_accepted
	@Test
	public void setParameterEndTurnAccepted() {
		assertTrue(newStep().setParameter(StepParameter.from(StepParameterKey.END_TURN, true)));
	}

	// rust: set_parameter_end_player_action_accepted
	@Test
	public void setParameterEndPlayerActionAccepted() {
		assertTrue(newStep().setParameter(StepParameter.from(StepParameterKey.END_PLAYER_ACTION, true)));
	}

	// rust: unrecognised_parameter_returns_false
	@Test
	public void unrecognisedParameterReturnsFalse() {
		assertFalse(newStep().setParameter(StepParameter.from(StepParameterKey.HOME_TEAM, true)));
	}
}
