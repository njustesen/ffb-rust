package com.fumbbl.ffb.server.inducements.bb2025.prayers;

import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.model.Game;
import com.fumbbl.ffb.model.Player;
import com.fumbbl.ffb.server.GameState;
import com.fumbbl.ffb.server.fixture.GameFixture;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import java.util.Collections;
import java.util.List;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirror of ffb-rust
 * crates/ffb-engine/src/inducements/bb2025/prayers/opponent_player_selector.rs tests.
 */
public class OpponentPlayerSelectorTest {

	private GameState gameState;
	private Game game;
	private OpponentPlayerSelector selector;

	@BeforeEach
	void setUp() {
		gameState = GameFixture.createGameState(3, RulesCollection.Rules.BB2025);
		game = gameState.getGame();
		selector = new OpponentPlayerSelector();
	}

	// rust: selects_opponent_team_players
	@Test
	public void selectsOpponentTeamPlayers() {
		List<Player<?>> selected = selector.selectPlayers(game.getTeamHome(), game, 3, Collections.emptySet());
		assertTrue(selected.size() > 0);
		assertTrue(selected.stream().allMatch(p -> p.getId().startsWith("away")));
	}

	// rust: selects_no_own_team_players
	@Test
	public void selectsNoOwnTeamPlayers() {
		List<Player<?>> selected = selector.selectPlayers(game.getTeamHome(), game, 6, Collections.emptySet());
		assertTrue(selected.stream().noneMatch(p -> p.getId().startsWith("home")));
	}

	// rust: selects_opponent_when_away_prays
	@Test
	public void selectsOpponentWhenAwayPrays() {
		List<Player<?>> selected = selector.selectPlayers(game.getTeamAway(), game, 3, Collections.emptySet());
		assertTrue(selected.size() > 0);
		assertTrue(selected.stream().allMatch(p -> p.getId().startsWith("home")));
	}

	// rust: instance_const_is_usable
	@Test
	public void instanceConstIsUsable() {
		gameState = GameFixture.createGameState(0, RulesCollection.Rules.BB2025);
		game = gameState.getGame();
		assertTrue(OpponentPlayerSelector.INSTANCE
			.selectPlayers(game.getTeamHome(), game, 1, Collections.emptySet()).isEmpty());
	}

	// rust: respects_nr_of_players_limit
	@Test
	public void respectsNrOfPlayersLimit() {
		List<Player<?>> selected = selector.selectPlayers(game.getTeamHome(), game, 2, Collections.emptySet());
		assertEquals(2, selected.size());
	}

	// rust: returns_empty_when_opponent_has_no_players
	@Test
	public void returnsEmptyWhenOpponentHasNoPlayers() {
		gameState = GameFixture.createGameState(0, RulesCollection.Rules.BB2025);
		game = gameState.getGame();
		assertTrue(selector.selectPlayers(game.getTeamHome(), game, 1, Collections.emptySet()).isEmpty());
	}
}
