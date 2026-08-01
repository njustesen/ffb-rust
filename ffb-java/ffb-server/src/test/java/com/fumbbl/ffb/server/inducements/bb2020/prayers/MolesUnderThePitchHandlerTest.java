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
 * crates/ffb-engine/src/inducements/bb2020/prayers/moles_under_the_pitch_handler.rs tests.
 */
public class MolesUnderThePitchHandlerTest {

	private GameState gameState;
	private Game game;
	private MolesUnderThePitchHandler handler;

	@BeforeEach
	void setUp() {
		gameState = GameFixture.createGameState(3, RulesCollection.Rules.BB2020);
		game = gameState.getGame();
		handler = new MolesUnderThePitchHandler();
	}

	// rust: handled_prayer_is_moles
	@Test
	public void handledPrayerIsMoles() {
		assertEquals(Prayer.MOLES_UNDER_THE_PITCH, handler.handledPrayer());
		assertTrue(handler.handles(Prayer.MOLES_UNDER_THE_PITCH));
	}

	// rust: init_effect_sets_prayer_state
	@Test
	public void initEffectSetsPrayerState() {
		assertTrue(handler.initEffect(gameState, game.getTeamHome()));
		assertTrue(gameState.getPrayerState().getMolesUnderThePitch().contains(game.getTeamHome().getId()));
	}

	// rust: remove_effect_clears_prayer_state
	@Test
	public void removeEffectClearsPrayerState() {
		gameState.getPrayerState().addMolesUnderThePitch(game.getTeamHome());
		handler.removeEffectInternal(gameState, game.getTeamHome());
		assertFalse(gameState.getPrayerState().getMolesUnderThePitch().contains(game.getTeamHome().getId()));
	}

	// rust: get_name_returns_handler_name
	@Test
	public void getNameReturnsHandlerName() {
		assertEquals("MolesUnderThePitchHandler", handler.getName());
	}
}
