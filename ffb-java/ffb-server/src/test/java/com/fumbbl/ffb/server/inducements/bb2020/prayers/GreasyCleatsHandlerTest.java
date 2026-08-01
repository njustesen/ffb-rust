package com.fumbbl.ffb.server.inducements.bb2020.prayers;

import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.inducement.bb2020.Prayer;
import com.fumbbl.ffb.model.AnimationType;
import com.fumbbl.ffb.model.Game;
import com.fumbbl.ffb.server.GameState;
import com.fumbbl.ffb.server.fixture.GameFixture;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirror of ffb-rust crates/ffb-engine/src/inducements/bb2020/prayers/greasy_cleats_handler.rs
 * tests.
 */
public class GreasyCleatsHandlerTest {

	private GameState gameState;
	private Game game;
	private GreasyCleatsHandler handler;

	@BeforeEach
	void setUp() {
		gameState = GameFixture.createGameState(3, RulesCollection.Rules.BB2020);
		game = gameState.getGame();
		handler = new GreasyCleatsHandler();
	}

	// rust: handles_prayer_greasy_cleats
	@Test
	public void handlesPrayerGreasyCleats() {
		assertEquals(Prayer.GREASY_CLEATS, handler.handledPrayer());
		assertTrue(handler.handles(Prayer.GREASY_CLEATS));
		assertFalse(handler.handles(Prayer.STILETTO));
	}

	// rust: animation_type_is_correct
	@Test
	public void animationTypeIsCorrect() {
		assertEquals(AnimationType.PRAYER_GREASY_CLEATS, handler.animationType());
	}

	// rust: init_effect_returns_true
	@Test
	public void initEffectReturnsTrue() {
		assertTrue(handler.initEffect(gameState, game.getTeamHome()));
	}

	// rust: remove_effect_is_callable
	@Test
	public void removeEffectIsCallable() {
		handler.removeEffectInternal(gameState, game.getTeamHome());
	}

	// rust: does_not_handle_other_prayers
	@Test
	public void doesNotHandleOtherPrayers() {
		assertFalse(handler.handles(Prayer.PERFECT_PASSING));
	}
}
