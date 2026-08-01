package com.fumbbl.ffb.server.step.mixed;

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
 * Mirror of ffb-rust crates/ffb-engine/src/step/mixed/step_quick_bite.rs (param subset). CATCHER_ID is
 * accepted via setParameter; unknown → false. The start / find-opponent / use-skill-command / report /
 * revert-end-turn tests dereference the catcher and are command/placement-driven — deferred.
 */
public class StepQuickBiteFixtureTest {

	private GameState gameState;

	@BeforeEach
	public void setUp() {
		gameState = GameFixture.createGameState(3, RulesCollection.Rules.BB2020);
	}

	private IStep newStep() {
		return GameFixture.createStep(gameState, StepId.QUICK_BITE);
	}

	// rust: set_parameter_catcher_id
	@Test
	public void setParameterCatcherIdAccepted() {
		assertTrue(newStep().setParameter(StepParameter.from(StepParameterKey.CATCHER_ID, "away1")));
	}

	// unrecognised parameter returns false
	@Test
	public void unknownParameterReturnsFalse() {
		assertFalse(newStep().setParameter(StepParameter.from(StepParameterKey.END_TURN, true)));
	}
}
