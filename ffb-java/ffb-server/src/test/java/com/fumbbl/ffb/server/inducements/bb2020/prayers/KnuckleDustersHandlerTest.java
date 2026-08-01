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
 * Mirror of ffb-rust crates/ffb-engine/src/inducements/bb2020/prayers/knuckle_dusters_handler.rs tests
 * (portable subset; the Rust init-effect grant tests are a documented headless divergence).
 */
public class KnuckleDustersHandlerTest {

	private GameState gameState;
	private Game game;
	private KnuckleDustersHandler handler;

	@BeforeEach
	void setUp() {
		gameState = GameFixture.createGameState(3, RulesCollection.Rules.BB2020);
		game = gameState.getGame();
		handler = new KnuckleDustersHandler();
	}

	// rust: handles_prayer_*
	@Test
	public void handlesPrayer() {
		assertEquals(Prayer.KNUCKLE_DUSTERS, handler.handledPrayer());
		assertTrue(handler.handles(Prayer.KNUCKLE_DUSTERS));
		assertFalse(handler.handles(Prayer.STILETTO));
	}

	// rust: animation_type_is_correct
	@Test
	public void animationTypeIsCorrect() {
		assertEquals(AnimationType.PRAYER_KNUCKLE_DUSTERS, handler.animationType());
	}

	// rust: init_effect_returns_true (no eligible players -> prayer wasted)
	@Test
	public void initEffectReturnsTrueWhenNoEligiblePlayers() {
		gameState = GameFixture.createGameState(0, RulesCollection.Rules.BB2020);
		game = gameState.getGame();
		assertTrue(handler.initEffect(gameState, game.getTeamHome()));
	}

	// rust: does_not_handle_other_prayers
	@Test
	public void doesNotHandleOtherPrayers() {
		assertFalse(handler.handles(Prayer.PERFECT_PASSING));
	}
}
