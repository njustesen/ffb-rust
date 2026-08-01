package com.fumbbl.ffb.server.step.phase.kickoff;

import com.fumbbl.ffb.FieldCoordinate;
import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.model.Game;
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
 * Mirror of ffb-rust crates/ffb-engine/src/step/phase/kickoff/step_kickoff_animation.rs. start() sets
 * the ball in play and returns NEXT_STEP; KICKED_PLAYER_COORDINATE / TOUCHBACK accepted via
 * setParameter; unknown → false. The without-touchback CATCH_KICKOFF publish + default kicking-square
 * (2,8 home / 27,8 away) tests are published-param / internal-field driven and deferred.
 */
public class StepKickoffAnimationFixtureTest {

	private GameState gameState;
	private Game game;

	@BeforeEach
	public void setUp() {
		gameState = GameFixture.createGameState(3, RulesCollection.Rules.BB2016);
		game = gameState.getGame();
	}

	private IStep newStep() {
		return GameFixture.createStep(gameState, StepId.KICKOFF_ANIMATION);
	}

	// rust: start_sets_ball_in_play
	@Test
	public void startSetsBallInPlay() {
		assertEquals(StepAction.NEXT_STEP, GameFixture.startStep(newStep()));
		assertTrue(game.getFieldModel().isBallInPlay());
	}

	// rust: set_parameter_kicking_player_coordinate
	@Test
	public void setParameterKickingPlayerCoordinateAccepted() {
		assertTrue(newStep().setParameter(StepParameter.from(StepParameterKey.KICKED_PLAYER_COORDINATE, new FieldCoordinate(13, 7))));
	}

	// rust: set_parameter_touchback
	@Test
	public void setParameterTouchbackAccepted() {
		assertTrue(newStep().setParameter(StepParameter.from(StepParameterKey.TOUCHBACK, true)));
	}

	// rust: set_parameter_unrecognized_returns_false
	@Test
	public void unknownParameterReturnsFalse() {
		assertFalse(newStep().setParameter(StepParameter.from(StepParameterKey.END_TURN, true)));
	}
}
