package com.fumbbl.ffb.server.step.bb2016.move_;

import com.fumbbl.ffb.PlayerAction;
import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.model.Game;
import com.fumbbl.ffb.server.GameState;
import com.fumbbl.ffb.server.fixture.GameFixture;
import com.fumbbl.ffb.server.step.IStep;
import com.fumbbl.ffb.server.step.StepAction;
import com.fumbbl.ffb.server.step.StepId;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

/**
 * Mirrors the Rust tests in
 * {@code ffb-rust/crates/ffb-engine/src/step/bb2016/move_/step_go_for_it.rs} (dice/param subset).
 * The Rust tests set step.roll directly; here the GFI d6 is preset via GameFixture.installScriptedDice.
 * The failure-publishes-end-turn and reroll-prompt/decline tests are deferred (published-param /
 * team-reroll-command state).
 */
public class StepGoForItFixtureTest {

	private GameState gameState;

	@BeforeEach
	public void setUp() {
		gameState = GameFixture.createGameState(3, RulesCollection.Rules.BB2016);
		GameFixture.placePlayer(gameState, "home1", 5, 5);
		GameFixture.setActingPlayer(gameState, "home1", PlayerAction.MOVE);
		Game game = gameState.getGame();
		game.getActingPlayer().setGoingForIt(true);
		game.getActingPlayer().setCurrentMove(10);
	}

	private IStep newStep() {
		return GameFixture.createStep(gameState, StepId.GO_FOR_IT);
	}

	// rust: success_on_roll_two_or_above_returns_next_step
	@Test
	public void successOnRollTwoOrAboveReturnsNextStep() {
		GameFixture.installScriptedDice(gameState, 2);
		assertEquals(StepAction.NEXT_STEP, GameFixture.startStep(newStep()));
	}

	// rust: failure_on_roll_one_goes_to_failure_label
	@Test
	public void failureOnRollOneGoesToFailureLabel() {
		gameState.getGame().getTurnDataHome().setReRolls(0);
		GameFixture.installScriptedDice(gameState, 1);
		assertEquals(StepAction.GOTO_LABEL, GameFixture.startStep(newStep()));
	}
}
