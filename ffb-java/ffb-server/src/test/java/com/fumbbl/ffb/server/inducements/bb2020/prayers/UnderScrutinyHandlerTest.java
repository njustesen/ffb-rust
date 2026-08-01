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
 * Mirror of ffb-rust crates/ffb-engine/src/inducements/bb2020/prayers/under_scrutiny_handler.rs tests.
 */
public class UnderScrutinyHandlerTest {

	private GameState gameState;
	private Game game;
	private UnderScrutinyHandler handler;

	@BeforeEach
	void setUp() {
		gameState = GameFixture.createGameState(3, RulesCollection.Rules.BB2020);
		game = gameState.getGame();
		handler = new UnderScrutinyHandler();
	}

	// rust: handled_prayer_is_under_scrutiny
	@Test
	public void handledPrayerIsUnderScrutiny() {
		assertEquals(Prayer.UNDER_SCRUTINY, handler.handledPrayer());
		assertTrue(handler.handles(Prayer.UNDER_SCRUTINY));
		assertFalse(handler.handles(Prayer.FOULING_FRENZY));
	}

	// rust: init_effect_scrutinises_opponent
	@Test
	public void initEffectScrutinisesOpponent() {
		assertTrue(handler.initEffect(gameState, game.getTeamHome()));
		assertTrue(gameState.getPrayerState().isUnderScrutiny(game.getTeamAway()));
		assertFalse(gameState.getPrayerState().isUnderScrutiny(game.getTeamHome()));
	}

	// rust: remove_effect_clears_opponent_scrutiny
	@Test
	public void removeEffectClearsOpponentScrutiny() {
		gameState.getPrayerState().addUnderScrutiny(game.getTeamAway());
		handler.removeEffectInternal(gameState, game.getTeamHome());
		assertFalse(gameState.getPrayerState().isUnderScrutiny(game.getTeamAway()));
	}

	// rust: get_name_returns_handler_name
	@Test
	public void getNameReturnsHandlerName() {
		assertEquals("UnderScrutinyHandler", handler.getName());
	}
}
