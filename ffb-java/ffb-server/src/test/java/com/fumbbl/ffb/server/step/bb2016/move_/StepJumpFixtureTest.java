package com.fumbbl.ffb.server.step.bb2016.move_;

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

/**
 * Mirrors the Rust tests in
 * {@code ffb-rust/crates/ffb-engine/src/step/bb2016/move_/step_jump.rs} (no-dice + success subset).
 * The Rust sets step.roll directly; here the jump AG d6 is preset via installScriptedDice. The
 * failure GOTO_LABEL (constructor-set label), publishes-Jumped, and reroll-prompt/decline tests are
 * deferred (goto-label constructor state / published-param / team-reroll command).
 */
public class StepJumpFixtureTest {

	private GameState gameState;

	@BeforeEach
	public void setUp() {
		gameState = GameFixture.createGameState(3, RulesCollection.Rules.BB2016);
		GameFixture.placePlayer(gameState, "home1", 5, 5);
		GameFixture.setActingPlayer(gameState, "home1", PlayerAction.MOVE);
	}

	private IStep newStep() {
		return GameFixture.createStep(gameState, StepId.JUMP);
	}

	// rust: not_jumping_returns_next_step
	@Test
	public void notJumpingReturnsNextStep() {
		gameState.getGame().getActingPlayer().setJumping(false);
		assertEquals(StepAction.NEXT_STEP, GameFixture.startStep(newStep()));
	}

	// rust: jumping_without_leap_skill_returns_next_step
	@Test
	public void jumpingWithoutLeapSkillReturnsNextStep() {
		gameState.getGame().getActingPlayer().setJumping(true);
		assertEquals(StepAction.NEXT_STEP, GameFixture.startStep(newStep()));
	}

	// rust: jumping_success_clears_jumping_and_publishes_jumped (success -> NEXT_STEP)
	@Test
	public void jumpingSuccessReturnsNextStep() {
		Game game = gameState.getGame();
		RosterPlayer p = (RosterPlayer) game.getPlayerById("home1");
		p.addSkill(GameFixture.skill(game, "Leap"));
		game.getActingPlayer().setJumping(true);
		GameFixture.installScriptedDice(gameState, 4); // AG 3, 4 >= target -> success
		assertEquals(StepAction.NEXT_STEP, GameFixture.startStep(newStep()));
	}
}
