package com.fumbbl.ffb.server.step.bb2016.move_;

import com.fumbbl.ffb.FieldCoordinate;
import com.fumbbl.ffb.PlayerAction;
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
 * Mirrors the Rust tests in
 * {@code ffb-rust/crates/ffb-engine/src/step/bb2016/move_/step_move_ball_and_chain.rs} (param +
 * non-ball-and-chain subset). COORDINATE_FROM / COORDINATE_TO are stored via setParameter. An acting
 * player without the Ball and Chain (movesRandomly) skill skips the random scatter and returns
 * NEXT_STEP. GOTO_LABEL_ON_END / GOTO_LABEL_ON_FALL_DOWN are init-consumed (setParameter false,
 * exempt); no_coordinate_from is exempt (Rust-defensive: Java dereferences the acting player's
 * skills with no null guard -> NPE). The out-of-bounds fall-down, occupied-target end, scatter-roll,
 * and scatter-player-report tests need the Ball and Chain skill + dice + placed target and are
 * deferred.
 */
public class StepMoveBallAndChainFixtureTest {

	private GameState gameState;

	@BeforeEach
	public void setUp() {
		gameState = GameFixture.createGameState(3, RulesCollection.Rules.BB2016);
		gameState.getGame().setHomePlaying(true);
		GameFixture.placePlayer(gameState, "home1", 5, 5);
		GameFixture.setActingPlayer(gameState, "home1", PlayerAction.MOVE);
	}

	private IStep newStep() {
		return GameFixture.createStep(gameState, StepId.MOVE_BALL_AND_CHAIN);
	}

	// rust: set_parameter_coordinate_from_accepted
	@Test
	public void setParameterCoordinateFromAccepted() {
		assertTrue(newStep().setParameter(StepParameter.from(StepParameterKey.COORDINATE_FROM, new FieldCoordinate(5, 5))));
	}

	// rust: set_parameter_coordinate_to_accepted
	@Test
	public void setParameterCoordinateToAccepted() {
		assertTrue(newStep().setParameter(StepParameter.from(StepParameterKey.COORDINATE_TO, new FieldCoordinate(6, 5))));
	}

	// rust: unrecognised_parameter_returns_false
	@Test
	public void unrecognisedParameterReturnsFalse() {
		assertFalse(newStep().setParameter(StepParameter.from(StepParameterKey.END_TURN, true)));
	}

	// rust: non_ball_and_chain_player_returns_next_step
	@Test
	public void nonBallAndChainPlayerReturnsNextStep() {
		IStep step = newStep();
		step.setParameter(StepParameter.from(StepParameterKey.COORDINATE_FROM, new FieldCoordinate(5, 5)));
		step.setParameter(StepParameter.from(StepParameterKey.COORDINATE_TO, new FieldCoordinate(6, 5)));
		assertEquals(StepAction.NEXT_STEP, GameFixture.startStep(step));
	}
}
