package com.fumbbl.ffb.server.inducements;

import com.fumbbl.ffb.CardTarget;
import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.inducement.Card;
import com.fumbbl.ffb.inducement.CardHandlerKey;
import com.fumbbl.ffb.inducement.InducementDuration;
import com.fumbbl.ffb.inducement.InducementPhase;
import com.fumbbl.ffb.model.Game;
import com.fumbbl.ffb.server.GameState;
import com.fumbbl.ffb.server.fixture.GameFixture;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirror of ffb-rust crates/ffb-engine/src/inducements/card_handler.rs base tests, using a
 * named test subclass (Java getName() returns the simple class name).
 */
public class CardHandlerTest {

	private GameState gameState;
	private Game game;

	static class TestCardHandler extends CardHandler {
		@Override
		protected CardHandlerKey handlerKey() {
			return com.fumbbl.ffb.inducement.bb2020.CardHandlerKey.WITCH_BREW;
		}
	}

	@BeforeEach
	void setUp() {
		gameState = GameFixture.createGameState(3, RulesCollection.Rules.BB2020);
		game = gameState.getGame();
	}

	private Card card(CardHandlerKey key) {
		return new Card("Test Card", "tc", com.fumbbl.ffb.inducement.bb2020.CardType.DIRTY_TRICK,
			CardTarget.ANY_PLAYER, false, new InducementPhase[0], InducementDuration.UNTIL_END_OF_GAME,
			"test", key);
	}

	// rust: is_responsible_matches_handler_key
	@Test
	public void isResponsibleMatchesHandlerKey() {
		TestCardHandler handler = new TestCardHandler();
		assertTrue(handler.isResponsible(card(com.fumbbl.ffb.inducement.bb2020.CardHandlerKey.WITCH_BREW)));
		assertFalse(handler.isResponsible(card(com.fumbbl.ffb.inducement.bb2020.CardHandlerKey.DISTRACT)));
		assertFalse(handler.isResponsible(card(null)));
	}

	// rust: allows_player_default_returns_true
	@Test
	public void allowsPlayerDefaultReturnsTrue() {
		assertTrue(new TestCardHandler().allowsPlayer(game,
			card(com.fumbbl.ffb.inducement.bb2020.CardHandlerKey.WITCH_BREW), game.getPlayerById("home1")));
	}

	// rust: get_name_returns_handler_name
	@Test
	public void getNameReturnsHandlerName() {
		assertEquals("TestCardHandler", new TestCardHandler().getName());
	}

	// rust: activate_on_game_default_returns_true
	@Test
	public void activateDefaultReturnsTrue() {
		assertTrue(new TestCardHandler().activate(
			card(com.fumbbl.ffb.inducement.bb2020.CardHandlerKey.WITCH_BREW), null, game.getPlayerById("home1")));
	}
}
