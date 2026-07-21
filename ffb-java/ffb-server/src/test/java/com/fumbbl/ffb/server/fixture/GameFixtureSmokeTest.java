package com.fumbbl.ffb.server.fixture;

import com.fumbbl.ffb.FieldCoordinate;
import com.fumbbl.ffb.PlayerAction;
import com.fumbbl.ffb.TurnMode;
import com.fumbbl.ffb.model.Game;
import com.fumbbl.ffb.model.RosterPlayer;
import com.fumbbl.ffb.server.GameState;

import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNotNull;
import static org.junit.jupiter.api.Assertions.assertTrue;

/** Self-test guarding the {@link GameFixture} helper surface used by the step-test port. */
public class GameFixtureSmokeTest {

	@Test
	public void helpersWork() {
		GameState gameState = GameFixture.createGameState(2);
		Game game = gameState.getGame();

		assertEquals(2, game.getTeamHome().getPlayers().length);
		assertNotNull(game.getPlayerById("home1"));
		assertNotNull(game.getPlayerById("away2"));
		assertEquals(6, game.getPlayerById("home1").getMovement());
		assertEquals(8, game.getPlayerById("home1").getArmour());

		RosterPlayer blocker = GameFixture.addPlayer(gameState, true, "hb1", 3, 5, 4, 4, 5, 9, "Block", "Dodge");
		assertEquals(4, blocker.getStrength());
		assertTrue(blocker.hasSkillExcludingTemporaryOnes(GameFixture.skill(game, "Block")));
		assertTrue(blocker.hasSkillExcludingTemporaryOnes(GameFixture.skill(game, "Dodge")));

		GameFixture.placePlayer(gameState, "hb1", 5, 5);
		assertEquals(new FieldCoordinate(5, 5), game.getFieldModel().getPlayerCoordinate(blocker));
		assertTrue(game.getFieldModel().getPlayerState(blocker).isStanding());

		GameFixture.setTurnMode(gameState, TurnMode.REGULAR);
		GameFixture.setHalf(gameState, 2);
		GameFixture.setActingPlayer(gameState, "hb1", PlayerAction.BLITZ);
		assertEquals(TurnMode.REGULAR, game.getTurnMode());
		assertEquals(2, game.getHalf());
		assertEquals("hb1", game.getActingPlayer().getPlayerId());
	}
}
