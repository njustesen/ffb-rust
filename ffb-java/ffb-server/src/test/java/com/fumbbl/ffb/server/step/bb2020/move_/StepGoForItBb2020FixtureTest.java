package com.fumbbl.ffb.server.step.bb2020.move_;

import com.fumbbl.ffb.PlayerAction;
import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.model.Game;
import com.fumbbl.ffb.model.RosterPlayer;
import com.fumbbl.ffb.server.GameState;
import com.fumbbl.ffb.server.fixture.GameFixture;
import com.fumbbl.ffb.server.step.IStep;
import com.fumbbl.ffb.server.step.StepAction;
import com.fumbbl.ffb.server.step.StepId;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirrors the Rust tests in
 * {@code ffb-rust/crates/ffb-engine/src/step/bb2020/move_/step_go_for_it.rs} (dice + blitz subset).
 * The Rust sets step.roll directly; here the GFI d6 is preset via installScriptedDice. A BLITZ
 * go-for-it sets blitzUsed and increments currentMove before the threshold check. The
 * failure-publishes-end-turn, reroll-prompt/decline, jumping-extra-move second-GFI, and
 * go-for-it-roll-report tests inspect published params, team rerolls, or reports and are deferred.
 */
public class StepGoForItBb2020FixtureTest {

	private GameState gameState;

	@BeforeEach
	public void setUp() {
		gameState = GameFixture.createGameState(3, RulesCollection.Rules.BB2020);
		gameState.getGame().setHomePlaying(true);
		GameFixture.placePlayer(gameState, "home1", 5, 5);
	}

	private IStep newStep() {
		return GameFixture.createStep(gameState, StepId.GO_FOR_IT);
	}

	// rust: success_on_roll_two_or_above_returns_next_step
	@Test
	public void successOnRollTwoOrAboveReturnsNextStep() {
		GameFixture.setActingPlayer(gameState, "home1", PlayerAction.MOVE);
		Game game = gameState.getGame();
		game.getActingPlayer().setGoingForIt(true);
		game.getActingPlayer().setCurrentMove(10);
		GameFixture.installScriptedDice(gameState, 2);
		assertEquals(StepAction.NEXT_STEP, GameFixture.startStep(newStep()));
	}

	// rust: failure_on_roll_one_goes_to_failure_label
	@Test
	public void failureOnRollOneGoesToFailureLabel() {
		GameFixture.setActingPlayer(gameState, "home1", PlayerAction.MOVE);
		Game game = gameState.getGame();
		game.getActingPlayer().setGoingForIt(true);
		game.getActingPlayer().setCurrentMove(10);
		game.getTurnDataHome().setReRolls(0);
		GameFixture.installScriptedDice(gameState, 1);
		assertEquals(StepAction.GOTO_LABEL, GameFixture.startStep(newStep()));
	}

	// rust: blitz_action_sets_blitz_used_and_increments_current_move
	@Test
	public void blitzActionSetsBlitzUsedAndIncrementsCurrentMove() {
		Game game = gameState.getGame();
		((RosterPlayer) game.getPlayerById("home1")).setMovement(4);
		GameFixture.setActingPlayer(gameState, "home1", PlayerAction.BLITZ);
		game.getActingPlayer().setCurrentMove(4);
		game.getActingPlayer().setGoingForIt(false);
		GameFixture.installScriptedDice(gameState, 4);
		assertEquals(StepAction.NEXT_STEP, GameFixture.startStep(newStep()));
		assertTrue(game.getTurnDataHome().isBlitzUsed());
		assertEquals(5, game.getActingPlayer().getCurrentMove());
	}
}
