package com.fumbbl.ffb.server.step.bb2025.pass;

import com.fumbbl.ffb.FieldCoordinate;
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
import static org.junit.jupiter.api.Assertions.assertNull;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirrors the Rust tests in
 * {@code ffb-rust/crates/ffb-engine/src/step/bb2025/pass/step_hand_over.rs} (next-step + field-state
 * subset). With no catcher the step still returns NEXT_STEP; it sets the ball moving and clears the
 * pass coordinate. The publishes-end-player-action / adjacent-catcher catch-mode / hand-over-report
 * tests inspect published parameters or reports and are deferred.
 */
public class StepHandOverBb2025FixtureTest {

	private GameState gameState;

	@BeforeEach
	public void setUp() {
		gameState = GameFixture.createGameState(3, RulesCollection.Rules.BB2025);
		gameState.getGame().setHomePlaying(true);
		GameFixture.placePlayer(gameState, "home1", 5, 5);
		GameFixture.setActingPlayer(gameState, "home1", PlayerAction.HAND_OVER);
		gameState.getGame().setThrowerId("home1");
	}

	private IStep newStep() {
		return GameFixture.createStep(gameState, StepId.HAND_OVER);
	}

	// rust: no_catcher_still_returns_next_step
	@Test
	public void noCatcherStillReturnsNextStep() {
		assertEquals(StepAction.NEXT_STEP, GameFixture.startStep(newStep()));
	}

	// rust: sets_ball_moving_and_clears_pass_coordinate
	@Test
	public void setsBallMovingAndClearsPassCoordinate() {
		Game game = gameState.getGame();
		game.setPassCoordinate(new FieldCoordinate(7, 7));
		GameFixture.startStep(newStep());
		assertTrue(game.getFieldModel().isBallMoving());
		assertNull(game.getPassCoordinate());
	}
}
