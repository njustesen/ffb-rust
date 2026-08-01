package com.fumbbl.ffb.server.util;

import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.model.Game;
import com.fumbbl.ffb.model.Player;
import com.fumbbl.ffb.server.GameState;
import com.fumbbl.ffb.server.fixture.GameFixture;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

/**
 * Mirror of ffb-rust crates/ffb-engine/src/util/server_util_player.rs findBlockStrength
 * assist-counting tests. An offensive assist (own-team, standing, adjacent to the defender) that
 * is not itself hindered by an adjacent opposing player adds +1 to the attacker's block strength.
 * (The Rust find_block_strength_simple(base, assists) additive helper has no Java counterpart —
 * it is a Rust-only convenience, exempt.)
 */
public class ServerUtilPlayerTest {

	private GameState gameState;
	private Game game;

	@BeforeEach
	void setUp() {
		gameState = GameFixture.createGameState(3, RulesCollection.Rules.BB2025);
		game = gameState.getGame();
	}

	private int blockStrength(int base) {
		return ServerUtilPlayer.findBlockStrength(game, game.getPlayerById("home1"), base,
			game.getPlayerById("away1"), false);
	}

	// rust: find_block_strength_no_players_returns_base (attacker vs isolated defender, no assists)
	@Test
	public void noAssistsReturnsBase() {
		GameFixture.placePlayer(gameState, "home1", 5, 7);
		GameFixture.placePlayer(gameState, "away1", 6, 7);
		assertEquals(3, blockStrength(3));
	}

	// rust: find_block_strength_standing_assist_counted (adjacent standing team-mate adds +1)
	@Test
	public void standingAssistCounted() {
		GameFixture.placePlayer(gameState, "home1", 5, 7);
		GameFixture.placePlayer(gameState, "away1", 6, 7);
		GameFixture.placePlayer(gameState, "home2", 6, 8); // adjacent to defender, not hindered
		assertEquals(4, blockStrength(3));
	}

	// rust: find_block_strength_prone_assist_not_counted (a prone team-mate has no tacklezone)
	@Test
	public void proneAssistNotCounted() {
		GameFixture.placePlayer(gameState, "home1", 5, 7);
		GameFixture.placePlayer(gameState, "away1", 6, 7);
		GameFixture.placePlayer(gameState, "home2", 6, 8);
		com.fumbbl.ffb.PlayerState ps = game.getFieldModel().getPlayerState(game.getPlayerById("home2"));
		game.getFieldModel().setPlayerState(game.getPlayerById("home2"),
			ps.changeBase(com.fumbbl.ffb.PlayerState.PRONE));
		assertEquals(3, blockStrength(3));
	}

	// rust: find_block_strength_hindered_assist_not_counted (an opposing player next to the assist
	// cancels it)
	@Test
	public void hinderedAssistNotCounted() {
		GameFixture.placePlayer(gameState, "home1", 5, 7);
		GameFixture.placePlayer(gameState, "away1", 6, 7);
		GameFixture.placePlayer(gameState, "home2", 6, 8);
		GameFixture.placePlayer(gameState, "away2", 7, 8); // hinders the assist
		assertEquals(3, blockStrength(3));
	}
}
