package com.fumbbl.ffb.server.step.bb2020.move_;

import com.fumbbl.ffb.FieldCoordinate;
import com.fumbbl.ffb.PlayerAction;
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
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirrors the Rust tests in
 * {@code ffb-rust/crates/ffb-engine/src/step/bb2020/move_/step_move_dodge.rs} (param + roll subset).
 * DODGE_ROLL is stored via setParameter; the step publishes DODGE_ROLL = rollSkill() back to itself so
 * installScriptedDice drives the outcome. GOTO_LABEL_ON_FAILURE / COORDINATE_FROM/TO are supplied for
 * start. The stand-firm-no-drop option, reroll-prompt/decline, and dodge-roll-report tests inspect
 * options, team rerolls, or reports and are deferred.
 */
public class StepMoveDodgeBb2020FixtureTest {

	private GameState gameState;

	@BeforeEach
	public void setUp() {
		gameState = GameFixture.createGameState(3, RulesCollection.Rules.BB2020);
		gameState.getGame().setHomePlaying(true);
		GameFixture.placePlayer(gameState, "home1", 5, 5);
		GameFixture.setActingPlayer(gameState, "home1", PlayerAction.MOVE);
	}

	private IStep newStep() {
		IStep step = GameFixture.createStep(gameState, StepId.MOVE_DODGE);
		step.setParameter(StepParameter.from(StepParameterKey.GOTO_LABEL_ON_FAILURE, "fail"));
		step.setParameter(StepParameter.from(StepParameterKey.COORDINATE_FROM, new FieldCoordinate(5, 5)));
		step.setParameter(StepParameter.from(StepParameterKey.COORDINATE_TO, new FieldCoordinate(6, 5)));
		return step;
	}

	// rust: set_parameter_dodge_roll_accepted
	@Test
	public void setParameterDodgeRollAccepted() {
		assertTrue(GameFixture.createStep(gameState, StepId.MOVE_DODGE)
			.setParameter(StepParameter.from(StepParameterKey.DODGE_ROLL, 4)));
	}

	// rust: not_dodging_returns_next_step
	@Test
	public void notDodgingReturnsNextStep() {
		gameState.getGame().getActingPlayer().setDodging(false);
		assertEquals(StepAction.NEXT_STEP, GameFixture.startStep(newStep()));
	}

	// rust: success_on_roll_two_or_above_returns_next_step
	@Test
	public void successOnRollTwoOrAboveReturnsNextStep() {
		gameState.getGame().getActingPlayer().setDodging(true);
		GameFixture.installScriptedDice(gameState, 6);
		assertEquals(StepAction.NEXT_STEP, GameFixture.startStep(newStep()));
	}

	// rust: failure_on_roll_one_goes_to_failure_label
	@Test
	public void failureOnRollOneGoesToFailureLabel() {
		Game game = gameState.getGame();
		game.getActingPlayer().setDodging(true);
		game.getTurnDataHome().setReRolls(0);
		GameFixture.installScriptedDice(gameState, 1);
		assertEquals(StepAction.GOTO_LABEL, GameFixture.startStep(newStep()));
	}
}
