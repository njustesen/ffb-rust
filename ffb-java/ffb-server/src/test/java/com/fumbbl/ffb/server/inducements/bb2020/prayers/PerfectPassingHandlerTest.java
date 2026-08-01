package com.fumbbl.ffb.server.inducements.bb2020.prayers;

import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.inducement.bb2020.Prayer;
import com.fumbbl.ffb.model.Game;
import com.fumbbl.ffb.server.GameState;
import com.fumbbl.ffb.server.fixture.GameFixture;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirror of ffb-rust crates/ffb-engine/src/inducements/bb2020/prayers/perfect_passing_handler.rs
 * tests.
 */
public class PerfectPassingHandlerTest {

	private GameState gameState;
	private Game game;
	private PerfectPassingHandler handler;

	@BeforeEach
	void setUp() {
		gameState = GameFixture.createGameState(3, RulesCollection.Rules.BB2020);
		game = gameState.getGame();
		handler = new PerfectPassingHandler();
	}

	// rust: handled_prayer_is_perfect_passing
	@Test
	public void handledPrayerIsPerfectPassing() {
		assertEquals(Prayer.PERFECT_PASSING, handler.handledPrayer());
		assertTrue(handler.handles(Prayer.PERFECT_PASSING));
		assertFalse(handler.handles(Prayer.FOULING_FRENZY));
	}

	// rust: init_effect_adds_completion_spp_team
	@Test
	public void initEffectAddsCompletionSppTeam() {
		assertTrue(handler.initEffect(gameState, game.getTeamHome()));
		assertTrue(gameState.getPrayerState().getAdditionalCompletionSppTeams().contains(game.getTeamHome().getId()));
	}

	// rust: remove_effect_clears_completion_spp_team
	@Test
	public void removeEffectClearsCompletionSppTeam() {
		gameState.getPrayerState().addGetAdditionalCompletionSpp(game.getTeamHome());
		handler.removeEffectInternal(gameState, game.getTeamHome());
		assertFalse(gameState.getPrayerState().getAdditionalCompletionSppTeams().contains(game.getTeamHome().getId()));
	}

	// rust: get_name_returns_handler_name
	@Test
	public void getNameReturnsHandlerName() {
		assertEquals("PerfectPassingHandler", handler.getName());
	}
}
