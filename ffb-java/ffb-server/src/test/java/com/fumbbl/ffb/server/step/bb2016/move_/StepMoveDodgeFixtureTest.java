package com.fumbbl.ffb.server.step.bb2016.move_;

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
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirrors the Rust tests in
 * {@code ffb-rust/crates/ffb-engine/src/step/bb2016/move_/step_move_dodge.rs} (param + roll subset).
 * The Rust sets step.dodge_roll directly; here the dodge d6 is preset via installScriptedDice (the
 * step publishes DODGE_ROLL = rollSkill() back onto itself). The publishes-steady-footing /
 * publishes-re-roll-used / publishes-break-tackle / reroll-prompt / stand-firm-option /
 * dodge-report / modifier-name tests inspect published params, options, reports or team rerolls and
 * are deferred.
 */
public class StepMoveDodgeFixtureTest {

	private GameState gameState;

	@BeforeEach
	public void setUp() {
		gameState = GameFixture.createGameState(3, RulesCollection.Rules.BB2016);
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

	// rust: set_parameter_goto_label_on_failure_accepted — EXEMPT: GOTO_LABEL_ON_FAILURE is
	// init-consumed in Java (stored during init(), but setParameter returns false afterwards),
	// whereas Rust's set_parameter returns true. Constructor-set-param divergence.

	// rust: unrecognised_parameter_returns_false
	@Test
	public void unrecognisedParameterReturnsFalse() {
		assertFalse(GameFixture.createStep(gameState, StepId.MOVE_DODGE)
			.setParameter(StepParameter.from(StepParameterKey.END_TURN, true)));
	}

	// rust: not_dodging_returns_next_step_immediately
	@Test
	public void notDodgingReturnsNextStep() {
		gameState.getGame().getActingPlayer().setDodging(false);
		assertEquals(StepAction.NEXT_STEP, GameFixture.startStep(newStep()));
	}

	// rust: success_on_roll_returns_next_step
	@Test
	public void successOnRollReturnsNextStep() {
		gameState.getGame().getActingPlayer().setDodging(true);
		GameFixture.installScriptedDice(gameState, 6);
		assertEquals(StepAction.NEXT_STEP, GameFixture.startStep(newStep()));
	}

	// rust: failure_goes_to_failure_label
	@Test
	public void failureGoesToFailureLabel() {
		Game game = gameState.getGame();
		game.getActingPlayer().setDodging(true);
		game.getTurnDataHome().setReRolls(0);
		GameFixture.installScriptedDice(gameState, 1);
		assertEquals(StepAction.GOTO_LABEL, GameFixture.startStep(newStep()));
	}
}
