package com.fumbbl.ffb.server.step.bb2016.pass;

import com.fumbbl.ffb.PlayerAction;
import com.fumbbl.ffb.RangeRuler;
import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.TurnMode;
import com.fumbbl.ffb.model.Game;
import com.fumbbl.ffb.server.GameState;
import com.fumbbl.ffb.server.fixture.GameFixture;
import com.fumbbl.ffb.server.fixture.GeneratorTestSupport;
import com.fumbbl.ffb.server.step.IStep;
import com.fumbbl.ffb.server.step.StepAction;
import com.fumbbl.ffb.server.step.StepId;
import com.fumbbl.ffb.server.step.StepParameter;
import com.fumbbl.ffb.server.step.StepParameterKey;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertNull;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirrors the Rust tests in
 * {@code ffb-rust/crates/ffb-engine/src/step/bb2016/pass/step_end_passing.rs} (param/branch subset).
 * The SPP-completion / passing-yards / interception-count tests are deferred — they require placed
 * passer/catcher/interceptor players, a ball at the pass coordinate, and SPP bookkeeping.
 */
public class StepEndPassingFixtureTest {

	private GameState gameState;

	@BeforeEach
	public void setUp() {
		gameState = GameFixture.createGameState(3, RulesCollection.Rules.BB2016);
		GameFixture.placePlayer(gameState, "home1", 5, 5);
		GameFixture.setActingPlayer(gameState, "home1", PlayerAction.PASS);
	}

	private IStep newStep() {
		return GameFixture.createStep(gameState, StepId.END_PASSING);
	}

	// rust: set_parameter_catcher_id
	@Test
	public void setParameterCatcherId() {
		assertTrue(newStep().setParameter(StepParameter.from(StepParameterKey.CATCHER_ID, "p1")));
	}

	// rust: set_parameter_pass_accurate
	@Test
	public void setParameterPassAccurate() {
		assertTrue(newStep().setParameter(StepParameter.from(StepParameterKey.PASS_ACCURATE, true)));
	}

	// rust: set_parameter_interceptor_id_none
	@Test
	public void setParameterInterceptorIdNone() {
		assertTrue(newStep().setParameter(StepParameter.from(StepParameterKey.INTERCEPTOR_ID, (String) null)));
	}

	// rust: clears_range_ruler_and_out_of_bounds
	@Test
	public void clearsRangeRulerAndOutOfBounds() {
		Game game = gameState.getGame();
		game.getFieldModel().setOutOfBounds(true);
		game.getFieldModel().setRangeRuler(new RangeRuler("p", null, 3, false));
		GameFixture.startStep(newStep());
		assertFalse(game.getFieldModel().isOutOfBounds());
		assertNull(game.getFieldModel().getRangeRuler());
	}

	// rust: bomb_turn_pushes_bomb_sequence
	@Test
	public void bombTurnPushesBombSequence() {
		gameState.getGame().setTurnMode(TurnMode.BOMB_HOME);
		assertEquals(StepAction.NEXT_STEP, GameFixture.startStep(newStep()));
		assertTrue(GeneratorTestSupport.sequence(gameState).length > 0);
	}

	// rust: animosity_retry_pushes_pass_sequence
	@Test
	public void animosityRetryPushesPassSequence() {
		gameState.getGame().getActingPlayer().setSufferingAnimosity(true);
		assertEquals(StepAction.NEXT_STEP, GameFixture.startStep(newStep()));
		assertTrue(GeneratorTestSupport.sequence(gameState).length > 0);
	}

	// rust: end_player_action_pushes_end_player_action_sequence
	@Test
	public void endPlayerActionPushesEndPlayerActionSequence() {
		IStep step = newStep();
		step.setParameter(StepParameter.from(StepParameterKey.END_PLAYER_ACTION, true));
		assertEquals(StepAction.NEXT_STEP, GameFixture.startStep(step));
		assertTrue(GeneratorTestSupport.sequence(gameState).length > 0);
	}
}
