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
 * Mirror of ffb-rust crates/ffb-engine/src/inducements/bb2020/prayers/bad_habits_handler.rs
 * tests.
 */
public class BadHabitsHandlerTest {

	private GameState gameState;
	private Game game;
	private BadHabitsHandler handler;

	@BeforeEach
	void setUp() {
		gameState = GameFixture.createGameState(3, RulesCollection.Rules.BB2020);
		game = gameState.getGame();
		handler = new BadHabitsHandler();
	}

	// rust: handled_prayer_is_bad_habits
	@Test
	public void handledPrayerIsBadHabits() {
		assertEquals(Prayer.BAD_HABITS, handler.handledPrayer());
		assertTrue(handler.handles(Prayer.BAD_HABITS));
	}

	// rust: does_not_handle_other_prayers
	@Test
	public void doesNotHandleOtherPrayers() {
		assertFalse(handler.handles(Prayer.PERFECT_PASSING));
	}

	// rust: animation_type_is_prayer_bad_habits
	@Test
	public void animationTypeIsPrayerBadHabits() {
		assertEquals(AnimationType.PRAYER_BAD_HABITS, handler.animationType());
	}

	// rust: init_effect_returns_true
	@Test
	public void initEffectReturnsTrue() {
		GameFixture.installScriptedDice(gameState, 1);
		assertTrue(handler.initEffect(gameState, game.getTeamHome()));
	}

	// rust: remove_effect_is_callable
	@Test
	public void removeEffectIsCallable() {
		handler.removeEffectInternal(gameState, game.getTeamHome());
	}
}
