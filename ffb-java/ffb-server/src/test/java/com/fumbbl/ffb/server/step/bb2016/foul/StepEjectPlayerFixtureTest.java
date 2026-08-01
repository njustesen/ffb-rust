package com.fumbbl.ffb.server.step.bb2016.foul;

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
 * Mirror of ffb-rust crates/ffb-engine/src/step/.../step_eject_player.rs (param subset). FOULER_HAS_BALL
 * / ARGUE_THE_CALL_SUCCESSFUL are accepted via setParameter; unknown → false. The hook-driven box
 * placement / END_TURN publish / scatter-ball / GOTO_LABEL tests need the acting player put into the box
 * and are deferred.
 */
public class StepEjectPlayerFixtureTest {

	private GameState gameState;

	@BeforeEach
	public void setUp() {
		gameState = GameFixture.createGameState(3, RulesCollection.Rules.BB2016);
	}

	private IStep newStep() {
		return GameFixture.createStep(gameState, StepId.EJECT_PLAYER);
	}

	// rust: FOULER_HAS_BALL accepted
	@Test
	public void setParameterFoulerHasBallAccepted() {
		assertTrue(newStep().setParameter(StepParameter.from(StepParameterKey.FOULER_HAS_BALL, true)));
	}

	// rust: ARGUE_THE_CALL_SUCCESSFUL accepted
	@Test
	public void setParameterArgueTheCallSuccessfulAccepted() {
		assertTrue(newStep().setParameter(StepParameter.from(StepParameterKey.ARGUE_THE_CALL_SUCCESSFUL, true)));
	}

	// unrecognised parameter returns false
	@Test
	public void unknownParameterReturnsFalse() {
		assertFalse(newStep().setParameter(StepParameter.from(StepParameterKey.END_TURN, true)));
	}
}
