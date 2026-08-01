package com.fumbbl.ffb.server.inducements.bb2025.prayers;

import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.inducement.bb2025.Prayer;
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
 * Mirror of ffb-rust crates/ffb-engine/src/inducements/bb2025/prayers/throw_a_rock_handler.rs
 * tests. NOTE: the Java BB2025 initEffect additionally registers a THROW_ROCK-usage inducement
 * on the praying team; the Rust handler documents this as unported (headless skips inducement
 * registration), so only the shared behaviors are mirrored here.
 */
public class ThrowARockHandlerTest {

	private GameState gameState;
	private Game game;
	private ThrowARockHandler handler;

	@BeforeEach
	void setUp() {
		gameState = GameFixture.createGameState(3, RulesCollection.Rules.BB2025);
		game = gameState.getGame();
		handler = new ThrowARockHandler();
	}

	// rust: handles_prayer_throw_a_rock
	@Test
	public void handlesPrayerThrowARock() {
		assertEquals(Prayer.THROW_A_ROCK, handler.handledPrayer());
		assertTrue(handler.handles(Prayer.THROW_A_ROCK));
		assertFalse(handler.handles(Prayer.IRON_MAN));
	}

	// rust: init_effect_returns_true
	@Test
	public void initEffectReturnsTrue() {
		assertTrue(handler.initEffect(gameState, game.getTeamHome()));
	}

	// rust: animation_type_is_correct
	@Test
	public void animationTypeIsCorrect() {
		assertEquals(AnimationType.PRAYER_THROW_A_ROCK, handler.animationType());
	}

	// rust: get_name_returns_handler_name
	@Test
	public void getNameReturnsHandlerName() {
		assertEquals("ThrowARockHandler", handler.getName());
	}
}
