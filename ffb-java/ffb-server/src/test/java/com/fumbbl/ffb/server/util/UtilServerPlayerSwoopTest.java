package com.fumbbl.ffb.server.util;

import com.fumbbl.ffb.FieldCoordinate;
import com.fumbbl.ffb.MoveSquare;
import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.model.Game;
import com.fumbbl.ffb.model.RosterPlayer;
import com.fumbbl.ffb.server.GameState;
import com.fumbbl.ffb.server.fixture.GameFixture;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

/**
 * Mirror of ffb-rust crates/ffb-engine/src/util/util_server_player_swoop.rs updateSwoopSquares
 * tests. The method clears existing move squares, then (only for a player with the Swoop
 * single-direction-scatter property, in bounds) adds the four orthogonally adjacent squares.
 * (The Rust add_swoop_square private-helper tests and defensive out-of-bounds/unknown-id no-panic
 * cases are Rust-structural — exempt.)
 */
public class UtilServerPlayerSwoopTest {

	private GameState gameState;
	private Game game;

	@BeforeEach
	void setUp() {
		gameState = GameFixture.createGameState(3, RulesCollection.Rules.BB2025);
		game = gameState.getGame();
	}

	// rust: update_swoop_squares_clears_existing_move_squares (no property -> cleared, none added)
	@Test
	public void clearsExistingMoveSquares() {
		GameFixture.placePlayer(gameState, "home1", 10, 7);
		game.getFieldModel().add(new MoveSquare(new FieldCoordinate(0, 0), 0, 0));
		game.getFieldModel().add(new MoveSquare(new FieldCoordinate(1, 1), 0, 0));
		UtilServerPlayerSwoop.updateSwoopSquares(gameState, game.getPlayerById("home1"));
		assertEquals(0, game.getFieldModel().getMoveSquares().length);
	}

	// rust: update_swoop_squares_no_property_results_in_empty_squares
	@Test
	public void noPropertyResultsInEmptySquares() {
		GameFixture.placePlayer(gameState, "home1", 10, 7);
		UtilServerPlayerSwoop.updateSwoopSquares(gameState, game.getPlayerById("home1"));
		assertEquals(0, game.getFieldModel().getMoveSquares().length);
	}

	// The Swoop (ttmScattersInSingleDirection) branch adds the four orthogonally adjacent squares.
	@Test
	public void swoopPropertyAddsFourAdjacentSquares() {
		GameFixture.placePlayer(gameState, "home1", 10, 7);
		((RosterPlayer) game.getPlayerById("home1")).addSkill(GameFixture.skill(game, "Swoop"));
		UtilServerPlayerSwoop.updateSwoopSquares(gameState, game.getPlayerById("home1"));
		assertEquals(4, game.getFieldModel().getMoveSquares().length);
	}
}
