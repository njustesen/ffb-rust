package com.fumbbl.ffb.server.inducements.mixed.prayers;

import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.model.AnimationType;
import com.fumbbl.ffb.model.Game;
import com.fumbbl.ffb.server.GameState;
import com.fumbbl.ffb.server.fixture.GameFixture;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirror of ffb-rust crates/ffb-engine/src/inducements/mixed/prayers/stiletto_handler.rs tests,
 * exercised through the concrete bb2020 subclass typed as the mixed class.
 */
public class StilettoHandlerTest {

	private GameState gameState;
	private Game game;
	private StilettoHandler handler;

	@BeforeEach
	void setUp() {
		gameState = GameFixture.createGameState(3, RulesCollection.Rules.BB2020);
		game = gameState.getGame();
		handler = new com.fumbbl.ffb.server.inducements.bb2020.prayers.StilettoHandler();
	}

	// rust: affected_players_is_one
	@Test
	public void affectedPlayersIsOne() {
		assertEquals(1, handler.affectedPlayers(gameState));
	}

	// rust: animation_type_is_correct
	@Test
	public void animationTypeIsCorrect() {
		assertEquals(AnimationType.PRAYER_STILETTO, handler.animationType());
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

	// rust: remove_effect_after_init_is_safe
	@Test
	public void removeEffectAfterInitIsSafe() {
		handler.initEffect(gameState, game.getTeamHome());
		handler.removeEffectInternal(gameState, game.getTeamHome());
	}
}
