package com.fumbbl.ffb.server.step.bb2016.move_;

import com.fumbbl.ffb.FieldCoordinate;
import com.fumbbl.ffb.PlayerAction;
import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.model.Game;
import com.fumbbl.ffb.model.Player;
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
 * {@code ffb-rust/crates/ffb-engine/src/step/bb2016/move_/step_move.rs} (param + move subset).
 * COORDINATE_FROM / COORDINATE_TO / MOVE_STACK are stored via setParameter. The
 * coordinate_to_is_none test is exempt (Rust guards a None target before dereferencing the acting
 * player; Java's executeStep has no such null guard). The ball-moves / rushing-stat / rooted-player /
 * move-square / go-for-it-at-MA tests need ball placement, a rooted state, or move-square inspection
 * and are deferred.
 */
public class StepMoveFixtureTest {

	private GameState gameState;

	@BeforeEach
	public void setUp() {
		gameState = GameFixture.createGameState(3, RulesCollection.Rules.BB2016);
		gameState.getGame().setHomePlaying(true);
		GameFixture.placePlayer(gameState, "home1", 5, 5);
		GameFixture.setActingPlayer(gameState, "home1", PlayerAction.MOVE);
	}

	private IStep moveStep(FieldCoordinate from, FieldCoordinate to) {
		IStep step = GameFixture.createStep(gameState, StepId.MOVE);
		step.setParameter(StepParameter.from(StepParameterKey.COORDINATE_FROM, from));
		step.setParameter(StepParameter.from(StepParameterKey.COORDINATE_TO, to));
		return step;
	}

	// rust: set_parameter_coordinate_from_accepted
	@Test
	public void setParameterCoordinateFromAccepted() {
		assertTrue(GameFixture.createStep(gameState, StepId.MOVE)
			.setParameter(StepParameter.from(StepParameterKey.COORDINATE_FROM, new FieldCoordinate(5, 5))));
	}

	// rust: set_parameter_coordinate_to_accepted
	@Test
	public void setParameterCoordinateToAccepted() {
		assertTrue(GameFixture.createStep(gameState, StepId.MOVE)
			.setParameter(StepParameter.from(StepParameterKey.COORDINATE_TO, new FieldCoordinate(6, 5))));
	}

	// rust: set_parameter_move_stack_accepted
	@Test
	public void setParameterMoveStackAccepted() {
		FieldCoordinate[] stack = { new FieldCoordinate(6, 5), new FieldCoordinate(7, 5), new FieldCoordinate(8, 5) };
		assertTrue(GameFixture.createStep(gameState, StepId.MOVE)
			.setParameter(StepParameter.from(StepParameterKey.MOVE_STACK, stack)));
	}

	// rust: unrecognised_parameter_returns_false
	@Test
	public void unrecognisedParameterReturnsFalse() {
		assertFalse(GameFixture.createStep(gameState, StepId.MOVE)
			.setParameter(StepParameter.from(StepParameterKey.END_TURN, true)));
	}

	// rust: move_updates_player_position
	@Test
	public void moveUpdatesPlayerPosition() {
		Game game = gameState.getGame();
		GameFixture.startStep(moveStep(new FieldCoordinate(5, 5), new FieldCoordinate(6, 5)));
		Player<?> player = game.getPlayerById("home1");
		assertEquals(new FieldCoordinate(6, 5), game.getFieldModel().getPlayerCoordinate(player));
	}

	// rust: move_increments_current_move_by_one
	@Test
	public void moveIncrementsCurrentMoveByOne() {
		Game game = gameState.getGame();
		game.getActingPlayer().setCurrentMove(2);
		GameFixture.startStep(moveStep(new FieldCoordinate(5, 5), new FieldCoordinate(6, 5)));
		assertEquals(3, game.getActingPlayer().getCurrentMove());
	}

	// rust: jumping_move_increments_current_move_by_two
	@Test
	public void jumpingMoveIncrementsCurrentMoveByTwo() {
		Game game = gameState.getGame();
		game.getActingPlayer().setJumping(true);
		game.getActingPlayer().setCurrentMove(2);
		GameFixture.startStep(moveStep(new FieldCoordinate(5, 5), new FieldCoordinate(7, 5)));
		assertEquals(4, game.getActingPlayer().getCurrentMove());
	}

	// rust: returns_next_step_on_successful_move
	@Test
	public void returnsNextStepOnSuccessfulMove() {
		assertEquals(StepAction.NEXT_STEP,
			GameFixture.startStep(moveStep(new FieldCoordinate(5, 5), new FieldCoordinate(6, 5))));
	}

	// rust: goes_for_it_false_when_well_within_ma
	@Test
	public void goesForItFalseWhenWellWithinMa() {
		Game game = gameState.getGame();
		game.getActingPlayer().setCurrentMove(0);
		GameFixture.startStep(moveStep(new FieldCoordinate(5, 5), new FieldCoordinate(6, 5)));
		assertFalse(game.getActingPlayer().isGoingForIt());
	}
}
