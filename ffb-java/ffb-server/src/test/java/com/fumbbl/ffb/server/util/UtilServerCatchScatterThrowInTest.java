package com.fumbbl.ffb.server.util;

import com.fumbbl.ffb.Direction;
import com.fumbbl.ffb.FieldCoordinate;
import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.model.Game;
import com.fumbbl.ffb.model.Player;
import com.fumbbl.ffb.model.RosterPlayer;
import com.fumbbl.ffb.server.GameState;
import com.fumbbl.ffb.server.fixture.GameFixture;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

/**
 * Mirror of ffb-rust crates/ffb-engine/src/util/util_server_catch_scatter_throw_in.rs tests.
 * findScatterCoordinate offsets a start coordinate by a direction * distance; findDivingCatchers
 * returns adjacent team-mates with the Diving Catch (canAttemptCatchInAdjacentSquares) skill.
 */
public class UtilServerCatchScatterThrowInTest {

	private GameState gameState;
	private Game game;

	@BeforeEach
	void setUp() {
		gameState = GameFixture.createGameState(3, RulesCollection.Rules.BB2025);
		game = gameState.getGame();
	}

	// rust: find_scatter_coordinate_north
	@Test
	public void findScatterCoordinateNorth() {
		assertEquals(new FieldCoordinate(5, 4),
			UtilServerCatchScatterThrowIn.findScatterCoordinate(new FieldCoordinate(5, 5), Direction.NORTH, 1));
	}

	// rust: find_scatter_coordinate_southeast
	@Test
	public void findScatterCoordinateSoutheast() {
		assertEquals(new FieldCoordinate(6, 6),
			UtilServerCatchScatterThrowIn.findScatterCoordinate(new FieldCoordinate(5, 5), Direction.SOUTHEAST, 1));
	}

	// rust: find_diving_catchers_empty_field
	@Test
	public void findDivingCatchersEmptyField() {
		Player<?>[] catchers = UtilServerCatchScatterThrowIn.findDivingCatchers(
			gameState, game.getTeamHome(), new FieldCoordinate(5, 5));
		assertEquals(0, catchers.length);
	}

	// rust: find_diving_catchers_finds_skilled_player
	@Test
	public void findDivingCatchersFindsSkilledPlayer() {
		GameFixture.placePlayer(gameState, "home1", 5, 4); // north of (5,5)
		((RosterPlayer) game.getPlayerById("home1")).addSkill(GameFixture.skill(game, "Diving Catch"));
		Player<?>[] catchers = UtilServerCatchScatterThrowIn.findDivingCatchers(
			gameState, game.getTeamHome(), new FieldCoordinate(5, 5));
		assertEquals(1, catchers.length);
		assertEquals("home1", catchers[0].getId());
	}
}
