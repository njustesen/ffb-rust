package com.fumbbl.ffb.server.inducements.bb2025.prayers;

import com.fumbbl.ffb.PlayerType;
import com.fumbbl.ffb.RulesCollection;
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
 * Mirror of ffb-rust crates/ffb-engine/src/inducements/bb2025/prayers/player_selector.rs tests.
 * BB2025 eligibility: RESERVE players only, star players always excluded.
 */
public class PlayerSelectorTest {

	private GameState gameState;
	private Game game;
	private PlayerSelector selector;

	@BeforeEach
	void setUp() {
		gameState = GameFixture.createGameState(3, RulesCollection.Rules.BB2025);
		game = gameState.getGame();
		selector = new PlayerSelector();
	}

	private boolean eligibleContains(String teamSide, String playerId) {
		List<Player<?>> eligible = selector.eligiblePlayers(
			"home".equals(teamSide) ? game.getTeamHome() : game.getTeamAway(), game, Collections.emptySet());
		return eligible.stream().anyMatch(p -> playerId.equals(p.getId()));
	}

	// rust: selects_reserve_regular_player
	@Test
	public void selectsReserveRegularPlayer() {
		assertTrue(eligibleContains("home", "home1"));
	}

	// rust: excludes_non_reserve_player
	@Test
	public void excludesNonReservePlayer() {
		GameFixture.placePlayer(gameState, "home1", 2, 2);
		assertFalse(eligibleContains("home", "home1"));
	}

	// rust: excludes_star_players
	@Test
	public void excludesStarPlayers() {
		((RosterPlayer) game.getPlayerById("home1")).setType(PlayerType.STAR);
		assertFalse(eligibleContains("home", "home1"));
	}

	// rust: respects_count_limit
	@Test
	public void respectsCountLimit() {
		List<Player<?>> selected = selector.selectPlayers(game.getTeamHome(), game, 2, Collections.emptySet());
		assertEquals(2, selected.size());
	}

	// rust: selects_from_correct_team
	@Test
	public void selectsFromCorrectTeam() {
		assertTrue(eligibleContains("away", "away1"));
		assertFalse(eligibleContains("away", "home1"));
	}
}
