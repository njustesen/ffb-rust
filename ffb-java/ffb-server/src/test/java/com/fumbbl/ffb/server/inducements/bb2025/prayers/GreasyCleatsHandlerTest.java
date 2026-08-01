package com.fumbbl.ffb.server.inducements.bb2025.prayers;

import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.inducement.bb2025.Prayer;
import com.fumbbl.ffb.model.AnimationType;
import com.fumbbl.ffb.model.Game;
import com.fumbbl.ffb.server.GameState;
import com.fumbbl.ffb.server.fixture.GameFixture;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import java.util.Arrays;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirror of ffb-rust crates/ffb-engine/src/inducements/bb2025/prayers/greasy_cleats_handler.rs
 * tests.
 */
public class GreasyCleatsHandlerTest {

	private GameState gameState;
	private Game game;
	private GreasyCleatsHandler handler;

	@BeforeEach
	void setUp() {
		gameState = GameFixture.createGameState(3, RulesCollection.Rules.BB2025);
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

	// rust: init_effect_returns_true
	@Test
	public void initEffectReturnsTrue() {
		assertTrue(handler.initEffect(gameState, game.getTeamHome()));
	}

	// rust: animation_type_is_correct
	@Test
	public void animationTypeIsCorrect() {
		assertEquals(AnimationType.PRAYER_GREASY_CLEATS, handler.animationType());
	}

	// rust: init_effect_selects_from_opponent_team
	@Test
	public void initEffectSelectsFromOpponentTeam() {
		handler.initEffect(gameState, game.getTeamHome());
		boolean anyAwayEnhanced = Arrays.stream(game.getTeamAway().getPlayers())
			.anyMatch(p -> p.hasActiveEnhancement(Prayer.GREASY_CLEATS.getName()));
		boolean anyHomeEnhanced = Arrays.stream(game.getTeamHome().getPlayers())
			.anyMatch(p -> p.hasActiveEnhancement(Prayer.GREASY_CLEATS.getName()));
		assertTrue(anyAwayEnhanced);
		assertFalse(anyHomeEnhanced);
	}

	// rust: does_not_handle_other_prayers
	@Test
	public void doesNotHandleOtherPrayers() {
		assertFalse(handler.handles(Prayer.PERFECT_PASSING));
	}
}
