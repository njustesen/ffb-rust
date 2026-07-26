package com.fumbbl.ffb.server.step.bb2025.move_;

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
 * {@code ffb-rust/crates/ffb-engine/src/step/bb2025/move_/step_go_for_it.rs} (dice subset). The Rust
 * sets step.roll directly; here the GFI d6 is preset via installScriptedDice. The failure-publishes-end-turn,
 * ball-and-chain-skip, blizzard-weather-modifier, reroll-prompt/accept/decline, and go-for-it-roll-report
 * tests inspect published params, weather, team rerolls, or reports and are deferred.
 */
public class StepGoForItBb2025FixtureTest {

	private GameState gameState;

	@BeforeEach
	public void setUp() {
		gameState = GameFixture.createGameState(3, RulesCollection.Rules.BB2025);
		gameState.getGame().setHomePlaying(true);
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
