package com.fumbbl.ffb.server.step.bb2016;

import com.fumbbl.ffb.FieldCoordinateBounds;
import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.kickoff.bb2016.KickoffResult;
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
 * Mirror of ffb-rust crates/ffb-engine/src/step/bb2016/step_apply_kickoff_result.rs (param subset).
 * KICKOFF_RESULT / TOUCHBACK / KICKOFF_BOUNDS are consumed via setParameter (return true); unrecognised
 * keys return false. The kickoff-result dispatch (which kickoff event to run) is hook/command driven and
 * deferred.
 */
public class StepApplyKickoffResultFixtureTest {

	private GameState gameState;

	@BeforeEach
	public void setUp() {
		gameState = GameFixture.createGameState(3, RulesCollection.Rules.BB2016);
	}

	private IStep newStep() {
		return GameFixture.createStep(gameState, StepId.APPLY_KICKOFF_RESULT);
	}

	// rust: set_parameter_kickoff_result_accepted
	@Test
	public void setParameterKickoffResultAccepted() {
		assertTrue(newStep().setParameter(StepParameter.from(StepParameterKey.KICKOFF_RESULT, KickoffResult.BLITZ)));
	}

	// rust: set_parameter_touchback_accepted
	@Test
	public void setParameterTouchbackAccepted() {
		assertTrue(newStep().setParameter(StepParameter.from(StepParameterKey.TOUCHBACK, true)));
	}

	// rust: KickoffBounds accepted (set_parameter matches on KickoffBounds → true)
	@Test
	public void setParameterKickoffBoundsAccepted() {
		assertTrue(newStep().setParameter(StepParameter.from(StepParameterKey.KICKOFF_BOUNDS, new FieldCoordinateBounds())));
	}

	// rust: set_parameter_unknown_returns_false
	@Test
	public void unknownParameterReturnsFalse() {
		assertFalse(newStep().setParameter(StepParameter.from(StepParameterKey.END_TURN, true)));
	}
}
