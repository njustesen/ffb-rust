package com.fumbbl.ffb.server.util;

import com.fumbbl.ffb.FieldCoordinate;
import com.fumbbl.ffb.PlayerState;
import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.TurnMode;
import com.fumbbl.ffb.model.Game;
import com.fumbbl.ffb.server.GameState;
import com.fumbbl.ffb.server.fixture.GameFixture;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertNotEquals;
import static org.junit.jupiter.api.Assertions.assertNull;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirror of ffb-rust crates/ffb-engine/src/util/util_server_setup.rs setupPlayer tests. During
 * setup the acting-team player is placed STANDING+active (RESERVE on a box coordinate); a
 * quick-snap move to a new square is active=false; occupied squares / unknown ids / wrong team
 * are rejected.
 */
public class UtilServerSetupTest {

	private GameState gameState;
	private Game game;

	@BeforeEach
	void setUp() {
		gameState = GameFixture.createGameState(3, RulesCollection.Rules.BB2025);
		game = gameState.getGame();
		game.setHomePlaying(true);
	}

	private PlayerState state(String id) {
		return game.getFieldModel().getPlayerState(game.getPlayerById(id));
	}

	// rust: setup_player_places_home_player_at_coordinate
	@Test
	public void placesHomePlayerAtCoordinate() {
		FieldCoordinate coord = new FieldCoordinate(10, 7);
		UtilServerSetup.setupPlayer(gameState, "home1", coord);
		assertEquals(coord, game.getFieldModel().getPlayerCoordinate(game.getPlayerById("home1")));
		assertEquals(PlayerState.STANDING, state("home1").getBase());
		assertTrue(state("home1").isActive());
	}

	// rust: setup_player_box_coordinate_sets_reserve
	@Test
	public void boxCoordinateSetsReserve() {
		FieldCoordinate box = new FieldCoordinate(FieldCoordinate.RSV_HOME_X, 5);
		assertTrue(box.isBoxCoordinate());
		UtilServerSetup.setupPlayer(gameState, "home1", box);
		assertEquals(PlayerState.RESERVE, state("home1").getBase());
	}

	// rust: setup_player_occupied_square_is_rejected
	@Test
	public void occupiedSquareIsRejected() {
		FieldCoordinate coord = new FieldCoordinate(10, 7);
		GameFixture.placePlayer(gameState, "home1", 10, 7);
		UtilServerSetup.setupPlayer(gameState, "home2", coord);
		// home2 stays in its reserve box (fixture reserve coord), never at the occupied square
		assertNotEquals(coord, game.getFieldModel().getPlayerCoordinate(game.getPlayerById("home2")));
		assertEquals(coord, game.getFieldModel().getPlayerCoordinate(game.getPlayerById("home1")));
	}

	// rust: setup_player_quick_snap_changed_coord_active_false
	@Test
	public void quickSnapChangedCoordActiveFalse() {
		GameFixture.setTurnMode(gameState, TurnMode.QUICK_SNAP);
		GameFixture.placePlayer(gameState, "home1", 5, 5);
		UtilServerSetup.setupPlayer(gameState, "home1", new FieldCoordinate(6, 5));
		assertEquals(PlayerState.STANDING, state("home1").getBase());
		assertFalse(state("home1").isActive());
	}

	// rust: setup_player_unknown_id_is_no_op
	@Test
	public void unknownIdIsNoOp() {
		UtilServerSetup.setupPlayer(gameState, "nobody", new FieldCoordinate(10, 7));
		// no exception, and no player occupies the square
		assertNull(game.getFieldModel().getPlayer(new FieldCoordinate(10, 7)));
	}

	// rust: setup_player_wrong_team_is_rejected (home not playing -> home player rejected)
	@Test
	public void wrongTeamIsRejected() {
		game.setHomePlaying(false);
		UtilServerSetup.setupPlayer(gameState, "home1", new FieldCoordinate(10, 7));
		assertNotEquals(new FieldCoordinate(10, 7),
			game.getFieldModel().getPlayerCoordinate(game.getPlayerById("home1")));
	}
}
