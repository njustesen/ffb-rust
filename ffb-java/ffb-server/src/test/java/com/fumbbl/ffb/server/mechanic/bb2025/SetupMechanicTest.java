package com.fumbbl.ffb.server.mechanic.bb2025;

import com.fumbbl.ffb.FieldCoordinate;
import com.fumbbl.ffb.PlayerState;
import com.fumbbl.ffb.model.Game;
import com.fumbbl.ffb.model.Player;
import com.fumbbl.ffb.model.RosterPlayer;
import com.fumbbl.ffb.option.GameOptionId;
import com.fumbbl.ffb.option.GameOptionInt;
import com.fumbbl.ffb.server.GameState;
import com.fumbbl.ffb.server.fixture.GameFixture;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirror of ffb-rust crates/ffb-engine/src/mechanic/bb2025/setup_mechanic.rs tests.
 * Empty-team checks use a 0-player fixture; pin tests place standing ACTIVE players
 * (pinPlayersInTacklezones only pins players whose state has the active flag).
 */
public class SetupMechanicTest {

	private final SetupMechanic mechanic = new SetupMechanic();

	private void placeActive(GameState gameState, String playerId, int x, int y) {
		Game game = gameState.getGame();
		Player<?> player = game.getPlayerById(playerId);
		game.getFieldModel().setPlayerCoordinate(player, new FieldCoordinate(x, y));
		game.getFieldModel().setPlayerState(player,
			new PlayerState(PlayerState.STANDING).changeActive(true));
	}

	private void setIntOption(Game game, GameOptionId optionId, int value) {
		GameOptionInt option = (GameOptionInt) game.getOptions().getFactory().createGameOption(optionId);
		option.setValue(value);
		game.getOptions().addOption(option);
	}

	// rust: check_setup_empty_team_is_valid
	@Test
	public void checkSetupEmptyTeamIsValid() {
		GameState gameState = GameFixture.createGameState(0);
		assertTrue(mechanic.checkSetup(gameState, true));
	}

	// rust: check_setup_with_swarmers_zero_matches_check_setup
	@Test
	public void checkSetupWithSwarmersZeroMatchesCheckSetup() {
		GameState gameState = GameFixture.createGameState(0);
		assertEquals(mechanic.checkSetup(gameState, true), mechanic.checkSetup(gameState, true, 0));
	}

	// rust: pin_players_in_tacklezones_no_players_is_noop
	@Test
	public void pinPlayersInTacklezonesNoPlayersIsNoop() {
		GameState gameState = GameFixture.createGameState(0);
		mechanic.pinPlayersInTacklezones(gameState, gameState.getGame().getTeamHome());
	}

	// rust: pin_players_in_tacklezones_chain_delegates
	@Test
	public void pinPlayersInTacklezonesChainDelegates() {
		GameState gameState = GameFixture.createGameState(0);
		mechanic.pinPlayersInTacklezones(gameState, gameState.getGame().getTeamHome(), true);
		mechanic.pinPlayersInTacklezones(gameState, gameState.getGame().getTeamHome(), false);
	}

	// rust: check_setup_uses_game_options
	@Test
	public void checkSetupUsesGameOptions() {
		GameState gameState = GameFixture.createGameState(0);
		Game game = gameState.getGame();
		setIntOption(game, GameOptionId.MAX_PLAYERS_ON_FIELD, 11);
		setIntOption(game, GameOptionId.MAX_PLAYERS_IN_WIDE_ZONE, 2);
		setIntOption(game, GameOptionId.MIN_PLAYERS_ON_LOS, 3);
		assertTrue(mechanic.checkSetup(gameState, true));
	}

	// rust: pin_players_pins_player_adjacent_to_opponent_with_tacklezone
	@Test
	public void pinPlayersPinsPlayerAdjacentToOpponentWithTacklezone() {
		GameState gameState = GameFixture.createGameState(3);
		Game game = gameState.getGame();
		placeActive(gameState, "home1", 5, 5);
		placeActive(gameState, "away1", 6, 5);
		mechanic.pinPlayersInTacklezones(gameState, game.getTeamHome(), false);
		assertFalse(game.getFieldModel().getPlayerState(game.getPlayerById("home1")).isActive(),
			"Player adjacent to opponent tackle zone should be pinned");
	}

	// rust: pin_players_does_not_pin_player_not_adjacent_to_opponent
	@Test
	public void pinPlayersDoesNotPinPlayerNotAdjacentToOpponent() {
		GameState gameState = GameFixture.createGameState(3);
		Game game = gameState.getGame();
		placeActive(gameState, "home1", 3, 3);
		placeActive(gameState, "away1", 10, 10);
		mechanic.pinPlayersInTacklezones(gameState, game.getTeamHome(), false);
		assertTrue(game.getFieldModel().getPlayerState(game.getPlayerById("home1")).isActive(),
			"Player not adjacent to any opponent should remain active");
	}

	// rust: pin_ball_and_chain_true_pins_moves_randomly_player
	@Test
	public void pinBallAndChainTruePinsMovesRandomlyPlayer() {
		GameState gameState = GameFixture.createGameState(3);
		Game game = gameState.getGame();
		((RosterPlayer) game.getPlayerById("home1")).addSkill(GameFixture.skill(game, "Ball and Chain"));
		placeActive(gameState, "home1", 5, 5);
		mechanic.pinPlayersInTacklezones(gameState, game.getTeamHome(), true);
		assertFalse(game.getFieldModel().getPlayerState(game.getPlayerById("home1")).isActive(),
			"BallAndChain player pinned when pinBallAndChain=true");
	}

	// rust: pin_ball_and_chain_false_does_not_pin_moves_randomly
	@Test
	public void pinBallAndChainFalseDoesNotPinMovesRandomly() {
		GameState gameState = GameFixture.createGameState(3);
		Game game = gameState.getGame();
		((RosterPlayer) game.getPlayerById("home1")).addSkill(GameFixture.skill(game, "Ball and Chain"));
		placeActive(gameState, "home1", 5, 5);
		mechanic.pinPlayersInTacklezones(gameState, game.getTeamHome(), false);
		assertTrue(game.getFieldModel().getPlayerState(game.getPlayerById("home1")).isActive(),
			"BallAndChain player should not be pinned when flag is false");
	}
}
