package com.fumbbl.ffb.server.step.action.ktm;

import com.fumbbl.ffb.PlayerState;
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
 * Mirror of ffb-rust crates/ffb-engine/src/step/action/ktm/step_kick_team_mate.rs (param subset).
 * KICKED_PLAYER_ID / KICKED_PLAYER_STATE / KICKED_PLAYER_HAS_BALL are consumed (setParameter returns
 * true). NR_OF_DICE is stored locally (clamped to 0..2) but the switch case `break`s so setParameter
 * returns FALSE — meaning a published NR_OF_DICE keeps propagating DOWN the stack to lower steps
 * (StepInitKickTeamMate publishes it). This true-vs-false distinction is the Rust Bug #14 fidelity fix.
 * Unrecognised keys return false. The kick roll / distance / scatter resolution is dice/hook driven and
 * deferred.
 */
public class StepKickTeamMateFixtureTest {

	private GameState gameState;

	@BeforeEach
	public void setUp() {
		gameState = GameFixture.createGameState(3, RulesCollection.Rules.BB2020);
	}

	private IStep newStep() {
		return GameFixture.createStep(gameState, StepId.KICK_TEAM_MATE);
	}

	// rust: KickedPlayerId consumed
	@Test
	public void setParameterKickedPlayerIdConsumed() {
		assertTrue(newStep().setParameter(StepParameter.from(StepParameterKey.KICKED_PLAYER_ID, "home2")));
	}

	// rust: KickedPlayerState consumed
	@Test
	public void setParameterKickedPlayerStateConsumed() {
		assertTrue(newStep().setParameter(
			StepParameter.from(StepParameterKey.KICKED_PLAYER_STATE, new PlayerState(PlayerState.STANDING))));
	}

	// rust: KickedPlayerHasBall consumed
	@Test
	public void setParameterKickedPlayerHasBallConsumed() {
		assertTrue(newStep().setParameter(StepParameter.from(StepParameterKey.KICKED_PLAYER_HAS_BALL, true)));
	}

	// rust: nr_of_dice_stored_but_not_consumed_so_it_keeps_propagating — NR_OF_DICE is stored but the
	// case `break`s → setParameter returns false so the parameter keeps flowing down the stack.
	@Test
	public void setParameterNrOfDiceStoredButNotConsumed() {
		assertFalse(newStep().setParameter(StepParameter.from(StepParameterKey.NR_OF_DICE, 2)));
	}

	// unrecognised parameter returns false
	@Test
	public void unknownParameterReturnsFalse() {
		assertFalse(newStep().setParameter(StepParameter.from(StepParameterKey.END_TURN, true)));
	}
}
