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

/**
 * Mirror of ffb-rust crates/ffb-engine/src/step/phase/kickoff/step_coin_choice.rs (guard subset).
 * With no coin choice yet made, start() shows the coin-choice dialog and waits (CONTINUE); the choice
 * arrives via a CLIENT_COIN_CHOICE command (not setParameter, so setParameter returns false). The
 * coin-throw / choosing-team publish / report tests are command+dice-driven and deferred.
 */
public class StepCoinChoiceFixtureTest {

	private GameState gameState;

	@BeforeEach
	public void setUp() {
		gameState = GameFixture.createGameState(3, RulesCollection.Rules.BB2016);
	}

	private IStep newStep() {
		return GameFixture.createStep(gameState, StepId.COIN_CHOICE);
	}

	// rust: start_without_choice_returns_cont
	@Test
	public void startWithoutChoiceReturnsCont() {
		assertEquals(StepAction.CONTINUE, GameFixture.startStep(newStep()));
	}

	// rust: set_parameter_returns_false
	@Test
	public void setParameterReturnsFalse() {
		assertFalse(newStep().setParameter(StepParameter.from(StepParameterKey.END_TURN, true)));
	}
}
