package com.fumbbl.ffb.server.step.phase.kickoff;

import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.server.GameState;
import com.fumbbl.ffb.server.fixture.GameFixture;
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
 * Mirror of ffb-rust crates/ffb-engine/src/step/phase/kickoff/step_touchback.rs (guard + param
 * subset). No touchback → NEXT_STEP; touchback without a chosen coordinate → CONTINUE (shows the
 * touchback dialog). TOUCHBACK is accepted via setParameter; unknown keys → false. The
 * with-coordinate resolution (ball placement + turn-mode REGULAR + catch/scatter) needs a placed
 * catcher and is deferred.
 */
public class StepTouchbackFixtureTest {

	private GameState gameState;

	@BeforeEach
	public void setUp() {
		gameState = GameFixture.createGameState(3, RulesCollection.Rules.BB2016);
	}

	private IStep newStep() {
		return GameFixture.createStep(gameState, StepId.TOUCHBACK);
	}

	// rust: no_touchback_returns_next_step_immediately
	@Test
	public void noTouchbackReturnsNextStepImmediately() {
		assertEquals(StepAction.NEXT_STEP, GameFixture.startStep(newStep()));
	}

	// rust: touchback_true_without_coordinate_returns_cont
	@Test
	public void touchbackTrueWithoutCoordinateReturnsCont() {
		IStep step = newStep();
		step.setParameter(StepParameter.from(StepParameterKey.TOUCHBACK, true));
		assertEquals(StepAction.CONTINUE, GameFixture.startStep(step));
	}

	// rust: set_parameter_touchback_accepted
	@Test
	public void setParameterTouchbackAccepted() {
		assertTrue(newStep().setParameter(StepParameter.from(StepParameterKey.TOUCHBACK, true)));
	}

	// rust: set_parameter_unrecognized_returns_false
	@Test
	public void setParameterUnrecognizedReturnsFalse() {
		assertFalse(newStep().setParameter(StepParameter.from(StepParameterKey.END_TURN, true)));
	}
}
