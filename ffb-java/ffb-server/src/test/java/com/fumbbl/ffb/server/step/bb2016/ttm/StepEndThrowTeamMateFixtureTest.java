package com.fumbbl.ffb.server.step.bb2016.ttm;

import com.fumbbl.ffb.FieldCoordinate;
import com.fumbbl.ffb.RangeRuler;
import com.fumbbl.ffb.RulesCollection;
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
import static org.junit.jupiter.api.Assertions.assertNull;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirrors the Rust tests in
 * {@code ffb-rust/crates/ffb-engine/src/step/bb2016/ttm/step_end_throw_team_mate.rs}.
 * The activate-player command test (CLIENT_ACTING_PLAYER -> Select + NEXT_STEP_AND_REPEAT) is
 * deferred — handleCommand returns a StepCommandStatus, not the resulting StepAction.
 */
public class StepEndThrowTeamMateFixtureTest {

	private GameState gameState;

	@BeforeEach
	public void setUp() {
		gameState = GameFixture.createGameState(3, RulesCollection.Rules.BB2016);
	}

	private IStep newStep() {
		return GameFixture.createStep(gameState, StepId.END_THROW_TEAM_MATE);
	}

	// rust: start_returns_next_step
	@Test
	public void startReturnsNextStep() {
		assertEquals(StepAction.NEXT_STEP, GameFixture.startStep(newStep()));
	}

	// rust: clears_pass_coordinate
	@Test
	public void clearsPassCoordinate() {
		Game game = gameState.getGame();
		game.setPassCoordinate(new FieldCoordinate(5, 5));
		GameFixture.startStep(newStep());
		assertNull(game.getPassCoordinate());
	}

	// rust: clears_range_ruler
	@Test
	public void clearsRangeRuler() {
		Game game = gameState.getGame();
		game.getFieldModel().setRangeRuler(new RangeRuler("p", null, 3, true));
		GameFixture.startStep(newStep());
		assertNull(game.getFieldModel().getRangeRuler());
	}

	// rust: set_parameter_end_turn
	@Test
	public void setParameterEndTurn() {
		assertTrue(newStep().setParameter(StepParameter.from(StepParameterKey.END_TURN, true)));
	}

	// rust: start_pushes_end_player_action_sequence
	@Test
	public void startPushesEndPlayerActionSequence() {
		GameFixture.startStep(newStep());
		assertTrue(GeneratorTestSupport.sequence(gameState).length > 0);
	}
}
