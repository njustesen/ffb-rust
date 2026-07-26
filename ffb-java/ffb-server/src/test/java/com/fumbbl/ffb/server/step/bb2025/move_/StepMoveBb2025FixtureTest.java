package com.fumbbl.ffb.server.step.bb2025.move_;

import com.fumbbl.ffb.FieldCoordinate;
import com.fumbbl.ffb.PlayerAction;
import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.model.Game;
import com.fumbbl.ffb.model.Player;
import com.fumbbl.ffb.server.GameState;
import com.fumbbl.ffb.server.fixture.GameFixture;
import com.fumbbl.ffb.server.step.IStep;
import com.fumbbl.ffb.server.step.StepId;
import com.fumbbl.ffb.server.step.StepParameter;
import com.fumbbl.ffb.server.step.StepParameterKey;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

/**
 * Mirrors the Rust tests in
 * {@code ffb-rust/crates/ffb-engine/src/step/bb2025/move_/step_move.rs} (move subset). The step moves
 * the acting player to COORDINATE_TO and increments current move by one (or two while jumping).
 * COORDINATE_FROM must be supplied (Java builds a TrackNumber from it). The no-acting-player,
 * publishes-entering-square, rooted, ball-movement and coordinate-to-null tests need published-param
 * inspection, a rooted state, ball placement, or are Rust-defensive and are deferred.
 */
public class StepMoveBb2025FixtureTest {

	private GameState gameState;

	@BeforeEach
	public void setUp() {
		gameState = GameFixture.createGameState(3, RulesCollection.Rules.BB2025);
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

	// rust: moves_player_to_coordinate_to
	@Test
	public void movesPlayerToCoordinateTo() {
		Game game = gameState.getGame();
		GameFixture.startStep(moveStep(new FieldCoordinate(5, 5), new FieldCoordinate(6, 5)));
		Player<?> player = game.getPlayerById("home1");
		assertEquals(new FieldCoordinate(6, 5), game.getFieldModel().getPlayerCoordinate(player));
	}

	// rust: increments_current_move_by_one_for_non_jumping
	@Test
	public void incrementsCurrentMoveByOneForNonJumping() {
		Game game = gameState.getGame();
		game.getActingPlayer().setCurrentMove(2);
		game.getActingPlayer().setJumping(false);
		GameFixture.startStep(moveStep(new FieldCoordinate(5, 5), new FieldCoordinate(6, 5)));
		assertEquals(3, game.getActingPlayer().getCurrentMove());
	}

	// rust: increments_current_move_by_two_for_jumping
	@Test
	public void incrementsCurrentMoveByTwoForJumping() {
		Game game = gameState.getGame();
		game.getActingPlayer().setCurrentMove(0);
		game.getActingPlayer().setJumping(true);
		GameFixture.startStep(moveStep(new FieldCoordinate(5, 5), new FieldCoordinate(7, 5)));
		assertEquals(2, game.getActingPlayer().getCurrentMove());
	}
}
