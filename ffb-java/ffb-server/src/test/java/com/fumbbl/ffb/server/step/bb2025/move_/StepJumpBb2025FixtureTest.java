package com.fumbbl.ffb.server.step.bb2025.move_;

import com.fumbbl.ffb.FieldCoordinate;
import com.fumbbl.ffb.PlayerAction;
import com.fumbbl.ffb.RulesCollection;
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
 * {@code ffb-rust/crates/ffb-engine/src/step/bb2025/move_/step_jump.rs} (param + jump subset).
 * MOVE_START is stored via setParameter; GOTO_LABEL_ON_FAILURE is init-consumed but stored for start.
 * The jump AG d6 is preset via installScriptedDice. The publishes-Jumped / coordinate-from /
 * reroll-prompt tests inspect published params or team rerolls and are deferred.
 */
public class StepJumpBb2025FixtureTest {

	private GameState gameState;

	@BeforeEach
	public void setUp() {
		gameState = GameFixture.createGameState(3, RulesCollection.Rules.BB2025);
		gameState.getGame().setHomePlaying(true);
		GameFixture.placePlayer(gameState, "home1", 5, 5);
		GameFixture.setActingPlayer(gameState, "home1", PlayerAction.MOVE);
	}

	private IStep newStep() {
		IStep step = GameFixture.createStep(gameState, StepId.JUMP);
		step.setParameter(StepParameter.from(StepParameterKey.GOTO_LABEL_ON_FAILURE, "fail"));
		return step;
	}

	private void addLeap() {
		((RosterPlayer) gameState.getGame().getPlayerById("home1")).addSkill(GameFixture.skill(gameState.getGame(), "Leap"));
	}

	// rust: set_parameter_move_start_accepted
	@Test
	public void setParameterMoveStartAccepted() {
		assertTrue(GameFixture.createStep(gameState, StepId.JUMP)
			.setParameter(StepParameter.from(StepParameterKey.MOVE_START, new FieldCoordinate(4, 4))));
	}

	// rust: not_jumping_returns_next_step
	@Test
	public void notJumpingReturnsNextStep() {
		gameState.getGame().getActingPlayer().setJumping(false);
		assertEquals(StepAction.NEXT_STEP, GameFixture.startStep(newStep()));
	}

	// rust: jumping_but_cannot_still_jump_returns_next_step
	@Test
	public void jumpingButCannotStillJumpReturnsNextStep() {
		gameState.getGame().getActingPlayer().setJumping(true);
		assertEquals(StepAction.NEXT_STEP, GameFixture.startStep(newStep()));
	}

	// rust: success_clears_jumping_flag
	@Test
	public void successClearsJumpingFlag() {
		Game game = gameState.getGame();
		addLeap();
		game.getActingPlayer().setJumping(true);
		GameFixture.installScriptedDice(gameState, 4);
		GameFixture.startStep(newStep());
		assertFalse(game.getActingPlayer().isJumping());
	}

	// rust: failure_goes_to_failure_label
	@Test
	public void failureGoesToFailureLabel() {
		Game game = gameState.getGame();
		addLeap();
		game.getActingPlayer().setJumping(true);
		game.getTurnDataHome().setReRolls(0);
		GameFixture.installScriptedDice(gameState, 1);
		assertEquals(StepAction.GOTO_LABEL, GameFixture.startStep(newStep()));
	}
}
