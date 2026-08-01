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
 * Mirror of ffb-rust crates/ffb-engine/src/inducements/bb2025/prayers/stiletto_handler.rs tests.
 * Stiletto uses the praying team's own PlayerSelector.
 */
public class StilettoHandlerTest {

	private GameState gameState;
	private Game game;
	private StilettoHandler handler;

	@BeforeEach
	void setUp() {
		gameState = GameFixture.createGameState(3, RulesCollection.Rules.BB2025);
		game = gameState.getGame();
		handler = new StilettoHandler();
	}

	// rust: handles_prayer_stiletto
	@Test
	public void handlesPrayerStiletto() {
		assertEquals(Prayer.STILETTO, handler.handledPrayer());
		assertTrue(handler.handles(Prayer.STILETTO));
		assertFalse(handler.handles(Prayer.GREASY_CLEATS));
	}

	// rust: init_effect_returns_true
	@Test
	public void initEffectReturnsTrue() {
		assertTrue(handler.initEffect(gameState, game.getTeamHome()));
	}

	// rust: animation_type_is_correct
	@Test
	public void animationTypeIsCorrect() {
		assertEquals(AnimationType.PRAYER_STILETTO, handler.animationType());
	}

	// rust: init_effect_selects_from_own_team
	@Test
	public void initEffectSelectsFromOwnTeam() {
		handler.initEffect(gameState, game.getTeamHome());
		boolean anyHomeEnhanced = Arrays.stream(game.getTeamHome().getPlayers())
			.anyMatch(p -> p.hasActiveEnhancement(Prayer.STILETTO.getName()));
		boolean anyAwayEnhanced = Arrays.stream(game.getTeamAway().getPlayers())
			.anyMatch(p -> p.hasActiveEnhancement(Prayer.STILETTO.getName()));
		assertTrue(anyHomeEnhanced);
		assertFalse(anyAwayEnhanced);
	}

	// rust: does_not_handle_other_prayers
	@Test
	public void doesNotHandleOtherPrayers() {
		assertFalse(handler.handles(Prayer.PERFECT_PASSING));
	}
}
