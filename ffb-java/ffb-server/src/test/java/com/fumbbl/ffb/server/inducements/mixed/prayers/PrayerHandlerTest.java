package com.fumbbl.ffb.server.inducements.mixed.prayers;

import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.inducement.Prayer;
import com.fumbbl.ffb.model.AnimationType;
import com.fumbbl.ffb.model.Game;
import com.fumbbl.ffb.model.Team;
import com.fumbbl.ffb.server.GameState;
import com.fumbbl.ffb.server.fixture.GameFixture;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirror of ffb-rust crates/ffb-engine/src/inducements/mixed/prayers/prayer_handler.rs base
 * tests, using a named test subclass (Java getName() returns the simple class name).
 */
public class PrayerHandlerTest {

	private GameState gameState;
	private Game game;

	static class TestPrayerHandler extends PrayerHandler {
		boolean removeInternalCalled = false;

		@Override
		public Prayer handledPrayer() {
			return com.fumbbl.ffb.inducement.bb2020.Prayer.FOULING_FRENZY;
		}

		@Override
		public AnimationType animationType() {
			return AnimationType.PRAYER_FOULING_FRENZY;
		}

		@Override
		public boolean initEffect(GameState gameState, Team prayingTeam) {
			return true;
		}

		@Override
		public void removeEffectInternal(GameState gameState, Team team) {
			removeInternalCalled = true;
		}
	}

	@BeforeEach
	void setUp() {
		gameState = GameFixture.createGameState(3, RulesCollection.Rules.BB2020);
		game = gameState.getGame();
	}

	// rust: handles_prayer_matches_by_name
	@Test
	public void handlesPrayerMatchesByName() {
		TestPrayerHandler handler = new TestPrayerHandler();
		assertTrue(handler.handles(com.fumbbl.ffb.inducement.bb2020.Prayer.FOULING_FRENZY));
		assertFalse(handler.handles(com.fumbbl.ffb.inducement.bb2020.Prayer.FRIENDS_WITH_THE_REF));
	}

	// rust: default_apply_selection_is_noop
	@Test
	public void defaultApplySelectionIsNoop() {
		new TestPrayerHandler().applySelection(game, new PrayerDialogSelection("home1", null));
	}

	// rust: get_name_returns_handler_name
	@Test
	public void getNameReturnsHandlerName() {
		assertEquals("TestPrayerHandler", new TestPrayerHandler().getName());
	}

	// rust: remove_effect_delegates_to_internal
	@Test
	public void removeEffectDelegatesToInternal() {
		TestPrayerHandler handler = new TestPrayerHandler();
		handler.removeEffect(gameState, game.getTeamHome());
		assertTrue(handler.removeInternalCalled);
	}
}
