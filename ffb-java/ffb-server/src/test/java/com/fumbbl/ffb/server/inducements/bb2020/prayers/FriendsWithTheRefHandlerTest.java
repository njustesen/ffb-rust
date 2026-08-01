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
 * crates/ffb-engine/src/inducements/bb2020/prayers/friends_with_the_ref_handler.rs tests.
 */
public class FriendsWithTheRefHandlerTest {

	private GameState gameState;
	private Game game;
	private FriendsWithTheRefHandler handler;

	@BeforeEach
	void setUp() {
		gameState = GameFixture.createGameState(3, RulesCollection.Rules.BB2020);
		game = gameState.getGame();
		handler = new FriendsWithTheRefHandler();
	}

	// rust: handled_prayer_is_friends_with_the_ref
	@Test
	public void handledPrayerIsFriendsWithTheRef() {
		assertEquals(Prayer.FRIENDS_WITH_THE_REF, handler.handledPrayer());
		assertTrue(handler.handles(Prayer.FRIENDS_WITH_THE_REF));
		assertFalse(handler.handles(Prayer.FAN_INTERACTION));
	}

	// rust: init_effect_sets_prayer_state
	@Test
	public void initEffectSetsPrayerState() {
		assertTrue(handler.initEffect(gameState, game.getTeamHome()));
		assertTrue(gameState.getPrayerState().isFriendsWithRef(game.getTeamHome()));
	}

	// rust: remove_effect_clears_prayer_state
	@Test
	public void removeEffectClearsPrayerState() {
		gameState.getPrayerState().addFriendsWithRef(game.getTeamHome());
		handler.removeEffectInternal(gameState, game.getTeamHome());
		assertFalse(gameState.getPrayerState().isFriendsWithRef(game.getTeamHome()));
	}

	// rust: get_name_returns_handler_name
	@Test
	public void getNameReturnsHandlerName() {
		assertEquals("FriendsWithTheRefHandler", handler.getName());
	}
}
