package com.fumbbl.ffb.server.step.phase.kickoff;

import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.server.GameState;
import com.fumbbl.ffb.server.fixture.GameFixture;
import com.fumbbl.ffb.server.step.IStep;
import com.fumbbl.ffb.server.step.StepId;
import com.fumbbl.ffb.server.step.StepParameter;
import com.fumbbl.ffb.server.step.StepParameterKey;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirror of ffb-rust crates/ffb-engine/src/step/phase/kickoff/step_kickoff_return.rs (param subset).
 * END_PLAYER_ACTION / END_TURN / TOUCHBACK are consumed via setParameter (return true — they only
 * additionally call consume() when the turn mode is KICKOFF_RETURN, which does not change the return
 * value); unrecognised keys return false. The kickoff-return move/command handling is command driven and
 * deferred.
 */
public class StepKickoffReturnFixtureTest {

	private GameState gameState;

	@BeforeEach
	public void setUp() {
		gameState = GameFixture.createGameState(3, RulesCollection.Rules.BB2020);
	}

	private IStep newStep() {
		return GameFixture.createStep(gameState, StepId.KICKOFF_RETURN);
	}

	// rust: set_parameter_end_player_action_accepted
	@Test
	public void setParameterEndPlayerActionAccepted() {
		assertTrue(newStep().setParameter(StepParameter.from(StepParameterKey.END_PLAYER_ACTION, true)));
	}

	// rust: set_parameter_end_turn_accepted
	@Test
	public void setParameterEndTurnAccepted() {
		assertTrue(newStep().setParameter(StepParameter.from(StepParameterKey.END_TURN, true)));
	}

	// rust: set_parameter_touchback_accepted
	@Test
	public void setParameterTouchbackAccepted() {
		assertTrue(newStep().setParameter(StepParameter.from(StepParameterKey.TOUCHBACK, true)));
	}

	// rust: set_parameter_unrecognized_returns_false
	@Test
	public void unknownParameterReturnsFalse() {
		assertFalse(newStep().setParameter(StepParameter.from(StepParameterKey.NR_OF_DICE, 2)));
	}
}
