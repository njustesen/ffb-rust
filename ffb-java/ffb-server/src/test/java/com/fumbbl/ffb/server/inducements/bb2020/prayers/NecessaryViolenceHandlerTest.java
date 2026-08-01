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
 * Mirror of ffb-rust
 * crates/ffb-engine/src/inducements/bb2020/prayers/necessary_violence_handler.rs tests.
 * BB2020-only handler (no mixed/bb2025 variants).
 */
public class NecessaryViolenceHandlerTest {

	private GameState gameState;
	private Game game;
	private NecessaryViolenceHandler handler;

	@BeforeEach
	void setUp() {
		gameState = GameFixture.createGameState(3, RulesCollection.Rules.BB2020);
		game = gameState.getGame();
		handler = new NecessaryViolenceHandler();
	}

	// rust: handled_prayer_is_necessary_violence
	@Test
	public void handledPrayerIsNecessaryViolence() {
		assertEquals(Prayer.NECESSARY_VIOLENCE, handler.handledPrayer());
		assertTrue(handler.handles(Prayer.NECESSARY_VIOLENCE));
		assertFalse(handler.handles(Prayer.PERFECT_PASSING));
	}

	// rust: init_effect_adds_cas_spp_team
	@Test
	public void initEffectAddsCasSppTeam() {
		assertTrue(handler.initEffect(gameState, game.getTeamHome()));
		assertTrue(gameState.getPrayerState().getAdditionalCasSppTeams().contains(game.getTeamHome().getId()));
	}

	// rust: remove_effect_clears_cas_spp_team
	@Test
	public void removeEffectClearsCasSppTeam() {
		gameState.getPrayerState().addGetAdditionalCasSpp(game.getTeamHome());
		handler.removeEffectInternal(gameState, game.getTeamHome());
		assertFalse(gameState.getPrayerState().getAdditionalCasSppTeams().contains(game.getTeamHome().getId()));
	}

	// rust: get_name_returns_handler_name
	@Test
	public void getNameReturnsHandlerName() {
		assertEquals("NecessaryViolenceHandler", handler.getName());
	}
}
