package com.fumbbl.ffb.server.inducements.mixed.prayers;

import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.model.Game;
import com.fumbbl.ffb.server.GameState;
import com.fumbbl.ffb.server.fixture.GameFixture;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertFalse;

/**
 * Mirror of ffb-rust
 * crates/ffb-engine/src/inducements/mixed/prayers/select_player_prayer_handler.rs tests
 * (portable subset). Java's initEffect creates a player-choice dialog and returns false while
 * waiting for the coach's selection.
 */
public class SelectPlayerPrayerHandlerTest {

	// rust: init_effect_returns_false_waiting_for_dialog
	@Test
	public void initEffectReturnsFalseWaitingForDialog() {
		GameState gameState = GameFixture.createGameState(3, RulesCollection.Rules.BB2020);
		Game game = gameState.getGame();
		SelectPlayerPrayerHandler handler =
			new com.fumbbl.ffb.server.inducements.bb2020.prayers.KnuckleDustersHandler();
		assertFalse(handler.initEffect(gameState, game.getTeamHome()));
	}
}
