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

import java.util.Arrays;
import java.util.Collections;
import java.util.List;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirror of ffb-rust
 * crates/ffb-engine/src/inducements/bb2020/prayers/opponent_player_selector.rs tests. The
 * opponent selector redirects determineTeam to the OTHER team.
 */
public class OpponentPlayerSelectorTest {

	private GameState gameState;
	private Game game;
	private OpponentPlayerSelector selector;

	@BeforeEach
	void setUp() {
		gameState = GameFixture.createGameState(3, RulesCollection.Rules.BB2020);
		game = gameState.getGame();
		selector = new OpponentPlayerSelector();
	}

	// rust: selects_from_opposing_team
	@Test
	public void selectsFromOpposingTeam() {
		List<Player<?>> selected = selector.selectPlayers(game.getTeamHome(), game, 3, Collections.emptySet());
		assertTrue(selected.size() > 0);
		assertTrue(selected.stream().allMatch(p -> p.getId().startsWith("away")));
	}

	// rust: opponent_selector_on_pitch_regular
	@Test
	public void opponentSelectorOnPitchRegular() {
		GameFixture.setTurnMode(gameState, TurnMode.REGULAR);
		GameFixture.placePlayer(gameState, "away1", 5, 5);
		List<Player<?>> selected = selector.selectPlayers(game.getTeamHome(), game, 3, Collections.emptySet());
		assertEquals(1, selected.size());
		assertEquals("away1", selected.get(0).getId());
	}

	// rust: opponent_selector_respects_count
	@Test
	public void opponentSelectorRespectsCount() {
		List<Player<?>> selected = selector.selectPlayers(game.getTeamHome(), game, 2, Collections.emptySet());
		assertEquals(2, selected.size());
	}

	// rust: opponent_selector_selects_from_home_when_away_prays
	@Test
	public void opponentSelectorSelectsFromHomeWhenAwayPrays() {
		List<Player<?>> selected = selector.selectPlayers(game.getTeamAway(), game, 3, Collections.emptySet());
		assertTrue(selected.size() > 0);
		assertTrue(selected.stream().allMatch(p -> p.getId().startsWith("home")));
	}

	// rust: opponent_selector_returns_empty_when_no_opponent_players
	@Test
	public void opponentSelectorReturnsEmptyWhenNoOpponentPlayers() {
		gameState = GameFixture.createGameState(0, RulesCollection.Rules.BB2020);
		game = gameState.getGame();
		assertTrue(selector.selectPlayers(game.getTeamHome(), game, 1, Collections.emptySet()).isEmpty());
	}

	// rust: opponent_selector_excludes_loner_players
	@Test
	public void opponentSelectorExcludesLonerPlayers() {
		Arrays.stream(game.getTeamAway().getPlayers())
			.forEach(p -> ((RosterPlayer) p).addSkill(GameFixture.skill(game, "Loner")));
		assertTrue(selector.selectPlayers(game.getTeamHome(), game, 3, Collections.emptySet()).isEmpty());
	}
}
