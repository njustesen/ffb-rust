package com.fumbbl.ffb.server.step.bb2016.ttm;

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
 * Mirror of ffb-rust crates/ffb-engine/src/step/bb2016/ttm/step_throw_team_mate.rs (param subset).
 * THROWN_PLAYER_ID / THROWN_PLAYER_STATE / THROWN_PLAYER_HAS_BALL accepted via setParameter; unknown →
 * false. The hook-driven throw resolution (accuracy roll / scatter / landing / reports) is
 * dice/command-driven and deferred.
 */
public class StepThrowTeamMateFixtureTest {

	private GameState gameState;

	@BeforeEach
	public void setUp() {
		gameState = GameFixture.createGameState(3, RulesCollection.Rules.BB2016);
	}

	private IStep newStep() {
		return GameFixture.createStep(gameState, StepId.THROW_TEAM_MATE);
	}

	// rust: THROWN_PLAYER_ID accepted
	@Test
	public void setParameterThrownPlayerIdAccepted() {
		assertTrue(newStep().setParameter(StepParameter.from(StepParameterKey.THROWN_PLAYER_ID, "home2")));
	}

	// rust: THROWN_PLAYER_STATE accepted
	@Test
	public void setParameterThrownPlayerStateAccepted() {
		assertTrue(newStep().setParameter(
			StepParameter.from(StepParameterKey.THROWN_PLAYER_STATE, new PlayerState(PlayerState.STANDING))));
	}

	// rust: THROWN_PLAYER_HAS_BALL accepted
	@Test
	public void setParameterThrownPlayerHasBallAccepted() {
		assertTrue(newStep().setParameter(StepParameter.from(StepParameterKey.THROWN_PLAYER_HAS_BALL, true)));
	}

	// unrecognised parameter returns false
	@Test
	public void unknownParameterReturnsFalse() {
		assertFalse(newStep().setParameter(StepParameter.from(StepParameterKey.END_TURN, true)));
	}
}
