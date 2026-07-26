package com.fumbbl.ffb.server.step.bb2016.pass;

import com.fumbbl.ffb.FieldCoordinate;
import com.fumbbl.ffb.RulesCollection;
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
 * {@code ffb-rust/crates/ffb-engine/src/step/bb2016/pass/step_missed_pass.rs} (param + next-step
 * subset). PASS_DEVIATES is stored via setParameter. The scatter branch scatters the ball from the
 * pass coordinate; the deviate branch deviates it from the thrower (both driven by scatter dice via
 * installScriptedDice) and return NEXT_STEP. The ball-moving / range-ruler / publishes-catch /
 * bomb-out-of-bounds / scatter-and-deviate-report tests inspect field state, published params, or
 * reports and are deferred.
 */
public class StepMissedPassFixtureTest {

	private GameState gameState;

	@BeforeEach
	public void setUp() {
		gameState = GameFixture.createGameState(3, RulesCollection.Rules.BB2016);
		gameState.getGame().setHomePlaying(true);
	}

	private IStep newStep() {
		return GameFixture.createStep(gameState, StepId.MISSED_PASS);
	}

	// rust: set_parameter_pass_deviates
	@Test
	public void setParameterPassDeviates() {
		assertTrue(newStep().setParameter(StepParameter.from(StepParameterKey.PASS_DEVIATES, true)));
	}

	// rust: set_parameter_unknown_returns_false
	@Test
	public void setParameterUnknownReturnsFalse() {
		assertFalse(newStep().setParameter(StepParameter.from(StepParameterKey.END_TURN, true)));
	}

	// rust: scatter_returns_next_step
	@Test
	public void scatterReturnsNextStep() {
		gameState.getGame().setPassCoordinate(new FieldCoordinate(10, 5));
		GameFixture.installScriptedDice(gameState, 1, 1, 1, 1, 1, 1, 1, 1);
		assertEquals(StepAction.NEXT_STEP, GameFixture.startStep(newStep()));
	}

	// rust: deviate_returns_next_step
	@Test
	public void deviateReturnsNextStep() {
		GameFixture.placePlayer(gameState, "home1", 10, 5);
		gameState.getGame().setThrowerId("home1");
		GameFixture.installScriptedDice(gameState, 1, 1, 1, 1, 1, 1);
		IStep step = newStep();
		step.setParameter(StepParameter.from(StepParameterKey.PASS_DEVIATES, true));
		assertEquals(StepAction.NEXT_STEP, GameFixture.startStep(step));
	}
}
