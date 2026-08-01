package com.fumbbl.ffb.server.step.action.pass;

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
 * Mirror of ffb-rust crates/ffb-engine/src/step/action/pass/step_animosity.rs (param subset). CATCHER_ID
 * is consumed via setParameter (return true); unrecognised keys return false. The animosity roll (race
 * mismatch check, re-roll, suffering flag) is dice/hook driven and unit-tested Rust-side; deferred here.
 */
public class StepAnimosityFixtureTest {

	private GameState gameState;

	@BeforeEach
	public void setUp() {
		gameState = GameFixture.createGameState(3, RulesCollection.Rules.BB2020);
	}

	private IStep newStep() {
		return GameFixture.createStep(gameState, StepId.ANIMOSITY);
	}

	// rust: catcher_id_parameter_accepted
	@Test
	public void setParameterCatcherIdAccepted() {
		assertTrue(newStep().setParameter(StepParameter.from(StepParameterKey.CATCHER_ID, "home2")));
	}

	// unrecognised parameter returns false
	@Test
	public void unknownParameterReturnsFalse() {
		assertFalse(newStep().setParameter(StepParameter.from(StepParameterKey.END_TURN, true)));
	}
}
