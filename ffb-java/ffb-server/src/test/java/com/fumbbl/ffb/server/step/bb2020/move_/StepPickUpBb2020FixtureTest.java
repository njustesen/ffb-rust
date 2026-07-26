package com.fumbbl.ffb.server.step.bb2020.move_;

import com.fumbbl.ffb.FieldCoordinate;
import com.fumbbl.ffb.PlayerAction;
import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.TurnMode;
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
 * Mirrors the Rust tests in
 * {@code ffb-rust/crates/ffb-engine/src/step/bb2020/move_/step_pick_up.rs} (ignore + roll subset).
 * FOLLOWUP_CHOICE(false) sets the ignore flag via setParameter, which makes the step skip to
 * NEXT_STEP. With the ball at the acting player, a successful pickup (installScriptedDice(6)) stops
 * the ball moving and continues; a failed pickup (installScriptedDice(1)) with no reroll gotos the
 * failure label. The reroll-prompt/decline and pickup-roll-report tests inspect team rerolls,
 * published params, or reports and are deferred.
 */
public class StepPickUpBb2020FixtureTest {

	private GameState gameState;

	@BeforeEach
	public void setUp() {
		gameState = GameFixture.createGameState(3, RulesCollection.Rules.BB2020);
		gameState.getGame().setHomePlaying(true);
		GameFixture.placePlayer(gameState, "home1", 5, 5);
		GameFixture.setActingPlayer(gameState, "home1", PlayerAction.MOVE);
	}

	private IStep newStep() {
		IStep step = GameFixture.createStep(gameState, StepId.PICK_UP);
		step.setParameter(StepParameter.from(StepParameterKey.GOTO_LABEL_ON_FAILURE, "fail"));
		return step;
	}

	private void ballAtActingPlayer() {
		Game game = gameState.getGame();
		game.getFieldModel().setBallCoordinate(new FieldCoordinate(5, 5));
		game.getFieldModel().setBallInPlay(true);
		game.getFieldModel().setBallMoving(true); // isPickUp requires the ball to be moving
	}

	// rust: set_parameter_followup_choice_false_sets_ignore
	@Test
	public void setParameterFollowupChoiceFalseSetsIgnore() {
		assertTrue(newStep().setParameter(StepParameter.from(StepParameterKey.FOLLOWUP_CHOICE, false)));
	}

	// rust: ignore_returns_next_step
	@Test
	public void ignoreReturnsNextStep() {
		IStep step = newStep();
		step.setParameter(StepParameter.from(StepParameterKey.FOLLOWUP_CHOICE, false));
		assertEquals(StepAction.NEXT_STEP, GameFixture.startStep(step));
	}

	// rust: success_sets_ball_not_moving
	@Test
	public void successSetsBallNotMoving() {
		ballAtActingPlayer();
		GameFixture.installScriptedDice(gameState, 6);
		assertEquals(StepAction.NEXT_STEP, GameFixture.startStep(newStep()));
		assertFalse(gameState.getGame().getFieldModel().isBallMoving());
	}

	// rust: failure_without_reroll_goes_to_label
	@Test
	public void failureWithoutRerollGoesToLabel() {
		Game game = gameState.getGame();
		game.setTurnMode(TurnMode.REGULAR);
		game.getTurnDataHome().setReRolls(0);
		ballAtActingPlayer();
		GameFixture.installScriptedDice(gameState, 1);
		assertEquals(StepAction.GOTO_LABEL, GameFixture.startStep(newStep()));
	}
}
