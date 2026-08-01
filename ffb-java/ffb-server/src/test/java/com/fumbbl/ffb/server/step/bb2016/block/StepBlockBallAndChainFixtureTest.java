package com.fumbbl.ffb.server.step.bb2016.block;

import com.fumbbl.ffb.PlayerAction;
import com.fumbbl.ffb.PlayerState;
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
 * Mirror of ffb-rust crates/ffb-engine/src/step/bb2016/block/step_block_ball_and_chain.rs (guard +
 * param subset). Only a Ball-and-Chain (movesRandomly) blocker with a prone/stunned OLD_DEFENDER_STATE
 * takes the pushback GOTO_LABEL path; anything else → NEXT_STEP. OLD_DEFENDER_STATE accepted via
 * setParameter; unknown → false. The pushback (GOTO_LABEL_ON_PUSHBACK is init-consumed) / old-state-
 * standing / init-pushback tests need the Ball-and-Chain skill + prone defender and are deferred.
 */
public class StepBlockBallAndChainFixtureTest {

	private GameState gameState;

	@BeforeEach
	public void setUp() {
		gameState = GameFixture.createGameState(3, RulesCollection.Rules.BB2016);
		GameFixture.placePlayer(gameState, "home1", 5, 5);
		GameFixture.setActingPlayer(gameState, "home1", PlayerAction.BLOCK);
	}

	private IStep newStep() {
		return GameFixture.createStep(gameState, StepId.BLOCK_BALL_AND_CHAIN);
	}

	// rust: no_property_returns_next (acting player without Ball-and-Chain → NEXT_STEP)
	@Test
	public void noPropertyReturnsNext() {
		assertEquals(StepAction.NEXT_STEP, GameFixture.startStep(newStep()));
	}

	// rust: set_parameter_old_defender_state
	@Test
	public void setParameterOldDefenderStateAccepted() {
		assertTrue(newStep().setParameter(
			StepParameter.from(StepParameterKey.OLD_DEFENDER_STATE, new PlayerState(PlayerState.STANDING))));
	}

	// unrecognised parameter returns false
	@Test
	public void unknownParameterReturnsFalse() {
		assertFalse(newStep().setParameter(StepParameter.from(StepParameterKey.END_TURN, true)));
	}
}
