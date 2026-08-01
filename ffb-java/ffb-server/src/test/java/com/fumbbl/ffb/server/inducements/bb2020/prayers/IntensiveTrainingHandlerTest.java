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
 * Mirror of ffb-rust crates/ffb-engine/src/inducements/bb2020/prayers/intensive_training_handler.rs
 * tests (portable subset; the Rust headless grant test is a documented divergence).
 */
public class IntensiveTrainingHandlerTest {

	private GameState gameState;
	private Game game;
	private IntensiveTrainingHandler handler;

	@BeforeEach
	void setUp() {
		gameState = GameFixture.createGameState(3, RulesCollection.Rules.BB2020);
		game = gameState.getGame();
		handler = new IntensiveTrainingHandler();
	}

	// rust: handles_prayer_intensive_training
	@Test
	public void handlesPrayerIntensiveTraining() {
		assertEquals(Prayer.INTENSIVE_TRAINING, handler.handledPrayer());
		assertTrue(handler.handles(Prayer.INTENSIVE_TRAINING));
		assertFalse(handler.handles(Prayer.IRON_MAN));
	}

	// rust: init_effect_returns_true (no eligible players -> prayer wasted)
	@Test
	public void initEffectReturnsTrueWhenNoEligiblePlayers() {
		gameState = GameFixture.createGameState(0, RulesCollection.Rules.BB2020);
		game = gameState.getGame();
		assertTrue(handler.initEffect(gameState, game.getTeamHome()));
	}

	// rust: animation_type_is_correct
	@Test
	public void animationTypeIsCorrect() {
		assertEquals(AnimationType.PRAYER_INTENSIVE_TRAINING, handler.animationType());
	}

	// rust: does_not_handle_other_prayers
	@Test
	public void doesNotHandleOtherPrayers() {
		assertFalse(handler.handles(Prayer.PERFECT_PASSING));
	}
}
