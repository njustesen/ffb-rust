package com.fumbbl.ffb.server.inducements.bb2020.prayers;

import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.TurnMode;
import com.fumbbl.ffb.model.Game;
import com.fumbbl.ffb.model.Player;
import com.fumbbl.ffb.model.RosterPlayer;
import com.fumbbl.ffb.server.GameState;
import com.fumbbl.ffb.server.fixture.GameFixture;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import java.util.Collections;
import java.util.List;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirror of ffb-rust crates/ffb-engine/src/inducements/bb2020/prayers/player_selector.rs tests.
 * BB2020 eligibility: RESERVE players during START_GAME, on-pitch players otherwise; players
 * with the hasToRollToUseTeamReroll property (Loner) are always excluded.
 */
public class PlayerSelectorTest {

	private GameState gameState;
	private Game game;
	private PlayerSelector selector;

	@BeforeEach
	void setUp() {
		gameState = GameFixture.createGameState(3, RulesCollection.Rules.BB2020);
		game = gameState.getGame();
		selector = new PlayerSelector();
	}

	private List<Player<?>> eligible() {
		return selector.eligiblePlayers(game.getTeamHome(), game, Collections.emptySet());
	}

	private boolean eligibleContains(String playerId) {
		return eligible().stream().anyMatch(p -> playerId.equals(p.getId()));
	}

	// rust: selects_reserve_player_at_start_game
	@Test
	public void selectsReservePlayerAtStartGame() {
		assertTrue(eligibleContains("home1"));
	}

	// rust: excludes_standing_player_at_start_game
	@Test
	public void excludesStandingPlayerAtStartGame() {
		GameFixture.placePlayer(gameState, "home1", 2, 2);
		assertFalse(eligibleContains("home1"));
	}

	// rust: selects_on_pitch_player_during_regular_play
	@Test
	public void selectsOnPitchPlayerDuringRegularPlay() {
		GameFixture.setTurnMode(gameState, TurnMode.REGULAR);
		GameFixture.placePlayer(gameState, "home1", 2, 2);
		assertTrue(eligibleContains("home1"));
	}

	// rust: excludes_off_pitch_player_during_regular_play
	@Test
	public void excludesOffPitchPlayerDuringRegularPlay() {
		GameFixture.setTurnMode(gameState, TurnMode.REGULAR);
		GameFixture.placePlayer(gameState, "home1", 2, 2);
		assertFalse(eligibleContains("home2"));
	}

	// rust: respects_count_limit
	@Test
	public void respectsCountLimit() {
		List<Player<?>> selected = selector.selectPlayers(game.getTeamHome(), game, 2, Collections.emptySet());
		assertEquals(2, selected.size());
	}

	// rust: excludes_loner_players
	@Test
	public void excludesLonerPlayers() {
		((RosterPlayer) game.getPlayerById("home1")).addSkill(GameFixture.skill(game, "Loner"));
		assertFalse(eligibleContains("home1"));
	}
}
