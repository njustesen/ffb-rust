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
 * Mirror of ffb-rust
 * crates/ffb-engine/src/inducements/bb2020/prayers/blessed_statue_of_nuffle_handler.rs tests
 * (portable subset; the Rust headless grant test is a documented divergence).
 */
public class BlessedStatueOfNuffleHandlerTest {

	private GameState gameState;
	private Game game;
	private BlessedStatueOfNuffleHandler handler;

	@BeforeEach
	void setUp() {
		gameState = GameFixture.createGameState(3, RulesCollection.Rules.BB2020);
		game = gameState.getGame();
		handler = new BlessedStatueOfNuffleHandler();
	}

	// rust: handles_prayer_blessed_statue_of_nuffle
	@Test
	public void handlesPrayerBlessedStatueOfNuffle() {
		assertEquals(Prayer.BLESSED_STATUE_OF_NUFFLE, handler.handledPrayer());
		assertTrue(handler.handles(Prayer.BLESSED_STATUE_OF_NUFFLE));
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
		assertEquals(AnimationType.PRAYER_BLESSED_STATUE_OF_NUFFLE, handler.animationType());
	}

	// rust: does_not_handle_other_prayers
	@Test
	public void doesNotHandleOtherPrayers() {
		assertFalse(handler.handles(Prayer.PERFECT_PASSING));
	}
}
