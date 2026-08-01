package com.fumbbl.ffb.server.inducements.bb2025.prayers;

import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.inducement.bb2025.Prayer;
import com.fumbbl.ffb.model.Game;
import com.fumbbl.ffb.server.GameState;
import com.fumbbl.ffb.server.fixture.GameFixture;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirror of ffb-rust crates/ffb-engine/src/inducements/bb2025/prayers/fouling_frenzy_handler.rs
 * tests.
 */
public class FoulingFrenzyHandlerTest {

	private GameState gameState;
	private Game game;
	private FoulingFrenzyHandler handler;

	@BeforeEach
	void setUp() {
		gameState = GameFixture.createGameState(3, RulesCollection.Rules.BB2025);
		game = gameState.getGame();
		handler = new FoulingFrenzyHandler();
	}

	// rust: handles_prayer_fouling_frenzy
	@Test
	public void handlesPrayerFoulingFrenzy() {
		assertEquals(Prayer.FOULING_FRENZY, handler.handledPrayer());
		assertTrue(handler.handles(Prayer.FOULING_FRENZY));
		assertFalse(handler.handles(Prayer.BAD_HABITS));
	}

	// rust: init_effect_adds_fouling_frenzy
	@Test
	public void initEffectAddsFoulingFrenzy() {
		assertTrue(handler.initEffect(gameState, game.getTeamHome()));
		assertTrue(gameState.getPrayerState().hasFoulingFrenzy(game.getTeamHome()));
	}

	// rust: remove_effect_clears_fouling_frenzy (via the public removeEffect wrapper)
	@Test
	public void removeEffectClearsFoulingFrenzy() {
		gameState.getPrayerState().addFoulingFrenzy(game.getTeamHome());
		handler.removeEffect(gameState, game.getTeamHome());
		assertFalse(gameState.getPrayerState().hasFoulingFrenzy(game.getTeamHome()));
	}

	// rust: get_name_returns_handler_name
	@Test
	public void getNameReturnsHandlerName() {
		assertEquals("FoulingFrenzyHandler", handler.getName());
	}
}
