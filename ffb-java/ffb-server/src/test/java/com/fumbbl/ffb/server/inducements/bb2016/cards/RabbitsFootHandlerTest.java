package com.fumbbl.ffb.server.inducements.bb2016.cards;

import com.fumbbl.ffb.CardTarget;
import com.fumbbl.ffb.PlayerState;
import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.inducement.Card;
import com.fumbbl.ffb.inducement.InducementDuration;
import com.fumbbl.ffb.inducement.InducementPhase;
import com.fumbbl.ffb.inducement.bb2016.CardHandlerKey;
import com.fumbbl.ffb.model.Game;
import com.fumbbl.ffb.model.Player;
import com.fumbbl.ffb.server.GameState;
import com.fumbbl.ffb.server.fixture.GameFixture;
import com.fumbbl.ffb.server.step.IStep;
import com.fumbbl.ffb.server.step.StepId;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirror of ffb-rust crates/ffb-engine/src/inducements/bb2016/cards/rabbits_foot_handler.rs tests.
 */
public class RabbitsFootHandlerTest {

	private GameState gameState;
	private Game game;
	private IStep step;
	private RabbitsFootHandler handler;

	@BeforeEach
	void setUp() {
		gameState = GameFixture.createGameState(3, RulesCollection.Rules.BB2016);
		game = gameState.getGame();
		step = GameFixture.createStep(gameState, StepId.INIT_START_GAME);
		handler = new RabbitsFootHandler();
	}

	private Card card(CardHandlerKey key) {
		return new Card("Test Card", "tc", com.fumbbl.ffb.inducement.bb2020.CardType.DIRTY_TRICK,
			CardTarget.ANY_PLAYER, false, new InducementPhase[0], InducementDuration.UNTIL_END_OF_GAME,
			"test", key);
	}

	// rust: is_responsible_for_correct_key
	@Test
	public void isResponsibleForCorrectKey() {
		assertTrue(handler.isResponsible(card(CardHandlerKey.RABBITS_FOOT)));
		assertFalse(handler.isResponsible(card(CardHandlerKey.CUSTARD_PIE)));
	}

	// rust: allows_player_true_for_player_without_prevent_property
	@Test
	public void allowsPlayerTrueForPlayerWithoutPreventProperty() {
		assertTrue(handler.allowsPlayer(game, card(CardHandlerKey.RABBITS_FOOT), game.getPlayerById("home1")));
	}

	// rust: activate_returns_true_by_default
	@Test
	public void activateReturnsTrueByDefault() {
		assertTrue(handler.activate(card(CardHandlerKey.RABBITS_FOOT), step, game.getPlayerById("home1")));
	}

	// rust: get_name_returns_struct_name
	@Test
	public void getNameReturnsHandlerName() {
		assertEquals("RabbitsFootHandler", handler.getName());
	}
}
