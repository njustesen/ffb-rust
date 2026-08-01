package com.fumbbl.ffb.server.inducements.bb2020.cards;

import com.fumbbl.ffb.CardEffect;
import com.fumbbl.ffb.CardTarget;
import com.fumbbl.ffb.PlayerState;
import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.inducement.Card;
import com.fumbbl.ffb.inducement.InducementDuration;
import com.fumbbl.ffb.inducement.InducementPhase;
import com.fumbbl.ffb.inducement.bb2020.CardHandlerKey;
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
 * Mirror of ffb-rust crates/ffb-engine/src/inducements/bb2020/cards/distract_handler.rs tests.
 * Distract marks opposing players within 3 squares as DISTRACTED; deactivation removes the
 * effect and clears confusion on affected players.
 */
public class DistractHandlerTest {

	private GameState gameState;
	private Game game;
	private IStep step;
	private DistractHandler handler;

	@BeforeEach
	void setUp() {
		gameState = GameFixture.createGameState(3, RulesCollection.Rules.BB2020);
		game = gameState.getGame();
		step = GameFixture.createStep(gameState, StepId.INIT_START_GAME);
		handler = new DistractHandler();
	}

	private Card card(CardHandlerKey key) {
		return new Card("Test Card", "tc", com.fumbbl.ffb.inducement.bb2020.CardType.DIRTY_TRICK,
			CardTarget.ANY_PLAYER, false, new InducementPhase[0], InducementDuration.UNTIL_END_OF_GAME,
			"test", key);
	}

	// rust: is_responsible_for_correct_key
	@Test
	public void isResponsibleForCorrectKey() {
		assertTrue(handler.isResponsible(card(CardHandlerKey.DISTRACT)));
		assertFalse(handler.isResponsible(card(CardHandlerKey.WITCH_BREW)));
	}

	// rust: activate_on_empty_field_returns_true (no adjacent opponents variant)
	@Test
	public void activateWithNoAdjacentOpponentsReturnsTrue() {
		GameFixture.placePlayer(gameState, "home1", 10, 7);
		assertTrue(handler.activate(card(CardHandlerKey.DISTRACT), step, game.getPlayerById("home1")));
		assertFalse(game.getFieldModel().hasCardEffect(game.getPlayerById("away1"), CardEffect.DISTRACTED));
	}

	// rust: activate_distracts_opponent_three_squares_away
	@Test
	public void activateDistractsOpponentThreeSquaresAway() {
		GameFixture.placePlayer(gameState, "home1", 10, 7);
		GameFixture.placePlayer(gameState, "away1", 13, 7);
		handler.activate(card(CardHandlerKey.DISTRACT), step, game.getPlayerById("home1"));
		assertTrue(game.getFieldModel().hasCardEffect(game.getPlayerById("away1"), CardEffect.DISTRACTED));
	}

	// rust: activate_does_not_distract_opponent_beyond_three_squares
	@Test
	public void activateDoesNotDistractOpponentBeyondThreeSquares() {
		GameFixture.placePlayer(gameState, "home1", 10, 7);
		GameFixture.placePlayer(gameState, "away1", 14, 7);
		handler.activate(card(CardHandlerKey.DISTRACT), step, game.getPlayerById("home1"));
		assertFalse(game.getFieldModel().hasCardEffect(game.getPlayerById("away1"), CardEffect.DISTRACTED));
	}

	// rust: deactivate_removes_distracted_effect
	@Test
	public void deactivateRemovesDistractedEffect() {
		Player<?> away1 = game.getPlayerById("away1");
		GameFixture.placePlayer(gameState, "away1", 5, 5);
		game.getFieldModel().addCardEffect(away1, CardEffect.DISTRACTED);
		handler.deactivate(card(CardHandlerKey.DISTRACT), step, null);
		assertFalse(game.getFieldModel().hasCardEffect(away1, CardEffect.DISTRACTED));
	}

	// rust: deactivate_clears_confused_on_distracted_player
	@Test
	public void deactivateClearsConfusedOnDistractedPlayer() {
		Player<?> away1 = game.getPlayerById("away1");
		GameFixture.placePlayer(gameState, "away1", 5, 5);
		game.getFieldModel().addCardEffect(away1, CardEffect.DISTRACTED);
		PlayerState state = game.getFieldModel().getPlayerState(away1);
		game.getFieldModel().setPlayerState(away1, state.changeConfused(true));
		handler.deactivate(card(CardHandlerKey.DISTRACT), step, null);
		assertFalse(game.getFieldModel().hasCardEffect(away1, CardEffect.DISTRACTED));
		assertFalse(game.getFieldModel().getPlayerState(away1).isConfused());
	}

	// rust: get_name_returns_struct_name
	@Test
	public void getNameReturnsHandlerName() {
		assertEquals("DistractHandler", handler.getName());
	}
}
