package com.fumbbl.ffb.server.step.action.foul;

import com.fumbbl.ffb.ApothecaryMode;
import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.server.GameState;
import com.fumbbl.ffb.server.InjuryResult;
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
 * Mirror of ffb-rust crates/ffb-engine/src/step/action/foul/step_referee.rs (param subset). INJURY_RESULT
 * is a CONDITIONAL accept: setParameter returns true only when the injury result's apothecary mode is
 * DEFENDER (storing it as the defender's result); any other mode (or an unrecognised key) returns false.
 * The eject/send-off resolution is hook driven and deferred; GOTO_LABEL_ON_END is init-supplied.
 */
public class StepRefereeFixtureTest {

	private GameState gameState;

	@BeforeEach
	public void setUp() {
		gameState = GameFixture.createGameState(3, RulesCollection.Rules.BB2020);
	}

	private IStep newStep() {
		return GameFixture.createStep(gameState, StepId.REFEREE);
	}

	private static InjuryResult injuryResult(ApothecaryMode mode) {
		InjuryResult injuryResult = new InjuryResult();
		injuryResult.injuryContext().setApothecaryMode(mode);
		return injuryResult;
	}

	// rust: InjuryResult with apothecaryMode == DEFENDER is accepted
	@Test
	public void setParameterInjuryResultDefenderAccepted() {
		assertTrue(newStep().setParameter(
			StepParameter.from(StepParameterKey.INJURY_RESULT, injuryResult(ApothecaryMode.DEFENDER))));
	}

	// rust: InjuryResult with a non-DEFENDER apothecary mode is rejected
	@Test
	public void setParameterInjuryResultNonDefenderRejected() {
		assertFalse(newStep().setParameter(
			StepParameter.from(StepParameterKey.INJURY_RESULT, injuryResult(ApothecaryMode.ATTACKER))));
	}

	// unrecognised parameter returns false
	@Test
	public void unknownParameterReturnsFalse() {
		assertFalse(newStep().setParameter(StepParameter.from(StepParameterKey.END_TURN, true)));
	}
}
