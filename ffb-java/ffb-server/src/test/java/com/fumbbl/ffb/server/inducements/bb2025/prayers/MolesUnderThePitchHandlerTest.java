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
 * crates/ffb-engine/src/inducements/bb2025/prayers/moles_under_the_pitch_handler.rs tests.
 */
public class MolesUnderThePitchHandlerTest {

	private GameState gameState;
	private Game game;
	private MolesUnderThePitchHandler handler;

	@BeforeEach
	void setUp() {
		gameState = GameFixture.createGameState(3, RulesCollection.Rules.BB2025);
		game = gameState.getGame();
		handler = new MolesUnderThePitchHandler();
	}

	// rust: handles_prayer_moles_under_the_pitch
	@Test
	public void handlesPrayerMolesUnderThePitch() {
		assertEquals(Prayer.MOLES_UNDER_THE_PITCH, handler.handledPrayer());
		assertTrue(handler.handles(Prayer.MOLES_UNDER_THE_PITCH));
		assertFalse(handler.handles(Prayer.FOULING_FRENZY));
	}

	// rust: init_effect_adds_moles
	@Test
	public void initEffectAddsMoles() {
		assertTrue(handler.initEffect(gameState, game.getTeamHome()));
		assertTrue(gameState.getPrayerState().getMolesUnderThePitch().contains(game.getTeamHome().getId()));
	}

	// rust: remove_effect_clears_moles (via the public removeEffect wrapper)
	@Test
	public void removeEffectClearsMoles() {
		gameState.getPrayerState().addMolesUnderThePitch(game.getTeamHome());
		handler.removeEffect(gameState, game.getTeamHome());
		assertFalse(gameState.getPrayerState().getMolesUnderThePitch().contains(game.getTeamHome().getId()));
	}

	// rust: get_name_returns_handler_name
	@Test
	public void getNameReturnsHandlerName() {
		assertEquals("MolesUnderThePitchHandler", handler.getName());
	}
}
