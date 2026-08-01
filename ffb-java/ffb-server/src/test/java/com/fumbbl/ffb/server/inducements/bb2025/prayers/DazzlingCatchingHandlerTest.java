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
 * Mirror of ffb-rust
 * crates/ffb-engine/src/inducements/bb2025/prayers/dazzling_catching_handler.rs tests.
 * BB2025-only prayer granting the "additional catches SPP" flag; removeEffectInternal is empty.
 * (The Rust handles_prayer_is_case_sensitive test is inexpressible in Java — handles() takes a
 * Prayer enum, not a string — exempt.)
 */
public class DazzlingCatchingHandlerTest {

	private GameState gameState;
	private Game game;
	private DazzlingCatchingHandler handler;

	@BeforeEach
	void setUp() {
		gameState = GameFixture.createGameState(3, RulesCollection.Rules.BB2025);
		game = gameState.getGame();
		handler = new DazzlingCatchingHandler();
	}

	// rust: handles_prayer_dazzling_catching
	@Test
	public void handlesPrayerDazzlingCatching() {
		assertEquals(Prayer.DAZZLING_CATCHING, handler.handledPrayer());
		assertTrue(handler.handles(Prayer.DAZZLING_CATCHING));
		assertFalse(handler.handles(Prayer.FOULING_FRENZY));
	}

	// rust: init_effect_adds_catches_spp_teams
	@Test
	public void initEffectAddsCatchesSpp() {
		assertTrue(handler.initEffect(gameState, game.getTeamHome()));
		assertTrue(gameState.getPrayerState().getAdditionalCatchesSppTeams().contains(game.getTeamHome().getId()));
	}

	// rust: remove_effect_is_noop (removeEffectInternal is empty — the entry must persist)
	@Test
	public void removeEffectInternalIsNoop() {
		gameState.getPrayerState().addGetAdditionalCatchesSpp(game.getTeamHome());
		handler.removeEffectInternal(gameState, game.getTeamHome());
		assertTrue(gameState.getPrayerState().getAdditionalCatchesSppTeams().contains(game.getTeamHome().getId()));
	}

	// rust: get_name_returns_handler_name
	@Test
	public void getNameReturnsHandlerName() {
		assertEquals("DazzlingCatchingHandler", handler.getName());
	}
}
