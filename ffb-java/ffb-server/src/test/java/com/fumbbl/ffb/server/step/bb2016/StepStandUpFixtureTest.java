package com.fumbbl.ffb.server.step.bb2016;

import com.fumbbl.ffb.PlayerAction;
import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.model.ActingPlayer;
import com.fumbbl.ffb.model.Game;
import com.fumbbl.ffb.model.RosterPlayer;
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
 * {@code ffb-rust/crates/ffb-engine/src/step/bb2016/step_stand_up.rs} (guard + roll subset).
 * The Rust sets step.roll directly; here the stand-up d6 is preset via installScriptedDice, and the
 * roll path is forced by lowering the acting player's movement below MINIMUM_MOVE_TO_STAND_UP (3).
 * GOTO_LABEL_ON_FAILURE is init-consumed (setParameter returns false) so that set_parameter test is
 * exempt; the reroll-prompt/accept/decline, blitz/foul/pass-used bookkeeping, and stand-up-report
 * tests are deferred (team-reroll command / turn-data-field / report inspection).
 */
public class StepStandUpFixtureTest {

	private GameState gameState;

	@BeforeEach
	public void setUp() {
		gameState = GameFixture.createGameState(3, RulesCollection.Rules.BB2016);
		gameState.getGame().setHomePlaying(true);
		GameFixture.placePlayer(gameState, "home1", 5, 5);
		GameFixture.setActingPlayer(gameState, "home1", PlayerAction.MOVE);
	}

	private IStep newStep() {
		IStep step = GameFixture.createStep(gameState, StepId.STAND_UP);
		step.setParameter(StepParameter.from(StepParameterKey.GOTO_LABEL_ON_FAILURE, "fail"));
		return step;
	}

	// rust: not_standing_up_returns_next_step_immediately
	@Test
	public void notStandingUpReturnsNextStep() {
		gameState.getGame().getActingPlayer().setStandingUp(false);
		assertEquals(StepAction.NEXT_STEP, GameFixture.startStep(newStep()));
	}

	// rust: already_moved_returns_next_step_immediately
	@Test
	public void alreadyMovedReturnsNextStep() {
		ActingPlayer actingPlayer = gameState.getGame().getActingPlayer();
		actingPlayer.setStandingUp(true);
		actingPlayer.setHasMoved(true);
		assertEquals(StepAction.NEXT_STEP, GameFixture.startStep(newStep()));
	}

	// rust: turn_started_set_to_true_on_execution
	@Test
	public void turnStartedSetToTrueOnExecution() {
		Game game = gameState.getGame();
		game.getActingPlayer().setStandingUp(true);
		game.getTurnData().setTurnStarted(false);
		GameFixture.startStep(newStep());
		assertTrue(game.getTurnData().isTurnStarted());
	}

	// rust: success_clears_standing_up_flag
	@Test
	public void successClearsStandingUpFlag() {
		Game game = gameState.getGame();
		((RosterPlayer) game.getPlayerById("home1")).setMovement(2); // MA < 3 forces the roll
		game.getActingPlayer().setStandingUp(true);
		GameFixture.installScriptedDice(gameState, 6); // >= 4 -> success
		assertEquals(StepAction.NEXT_STEP, GameFixture.startStep(newStep()));
		assertFalse(game.getActingPlayer().isStandingUp());
	}

	// rust: failure_goes_to_failure_label_with_end_player_action
	@Test
	public void failureGoesToFailureLabel() {
		Game game = gameState.getGame();
		((RosterPlayer) game.getPlayerById("home1")).setMovement(2); // MA < 3 forces the roll
		game.getActingPlayer().setStandingUp(true);
		game.getTurnDataHome().setReRolls(0);
		GameFixture.installScriptedDice(gameState, 1); // < 4 -> failure
		assertEquals(StepAction.GOTO_LABEL, GameFixture.startStep(newStep()));
	}
}
