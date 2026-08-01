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
 * Mirror of ffb-rust crates/ffb-engine/src/inducements/bb2020/prayers/fan_interaction_handler.rs tests.
 */
public class FanInteractionHandlerTest {

	private GameState gameState;
	private Game game;
	private FanInteractionHandler handler;

	@BeforeEach
	void setUp() {
		gameState = GameFixture.createGameState(3, RulesCollection.Rules.BB2020);
		game = gameState.getGame();
		handler = new FanInteractionHandler();
	}

	// rust: handled_prayer_is_fan_interaction
	@Test
	public void handledPrayerIsFanInteraction() {
		assertEquals(Prayer.FAN_INTERACTION, handler.handledPrayer());
		assertTrue(handler.handles(Prayer.FAN_INTERACTION));
		assertFalse(handler.handles(Prayer.FOULING_FRENZY));
	}

	// rust: init_effect_sets_prayer_state
	@Test
	public void initEffectSetsPrayerState() {
		assertTrue(handler.initEffect(gameState, game.getTeamHome()));
		assertTrue(gameState.getPrayerState().hasFanInteraction(game.getTeamHome()));
	}

	// rust: remove_effect_clears_prayer_state
	@Test
	public void removeEffectClearsPrayerState() {
		gameState.getPrayerState().addFanInteraction(game.getTeamHome());
		handler.removeEffectInternal(gameState, game.getTeamHome());
		assertFalse(gameState.getPrayerState().hasFanInteraction(game.getTeamHome()));
	}

	// rust: get_name_returns_handler_name
	@Test
	public void getNameReturnsHandlerName() {
		assertEquals("FanInteractionHandler", handler.getName());
	}
}
