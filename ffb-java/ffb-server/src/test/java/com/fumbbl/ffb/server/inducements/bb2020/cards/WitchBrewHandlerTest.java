package com.fumbbl.ffb.server.inducements.bb2020.cards;

import com.fumbbl.ffb.CardEffect;
import com.fumbbl.ffb.CardTarget;
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
 * Mirror of ffb-rust crates/ffb-engine/src/inducements/bb2020/cards/witch_brew_handler.rs tests.
 * Witch brew rolls a d6: 1 = Mad Cap Mushroom Potion, 2 = no effect, 3-6 = Sedative
 * (DiceInterpreter.interpretWitchBrewRoll).
 */
public class WitchBrewHandlerTest {

	private GameState gameState;
	private Game game;
	private IStep step;
	private WitchBrewHandler handler;
	private Player<?> player;

	@BeforeEach
	void setUp() {
		gameState = GameFixture.createGameState(3, RulesCollection.Rules.BB2020);
		game = gameState.getGame();
		step = GameFixture.createStep(gameState, StepId.INIT_START_GAME);
		handler = new WitchBrewHandler();
		player = game.getPlayerById("home1");
	}

	private Card card(CardHandlerKey key) {
		return new Card("Test Card", "tc", com.fumbbl.ffb.inducement.bb2020.CardType.DIRTY_TRICK,
			CardTarget.ANY_PLAYER, false, new InducementPhase[0], InducementDuration.UNTIL_END_OF_GAME,
			"test", key);
	}

	// rust: is_responsible_for_correct_key
	@Test
	public void isResponsibleForCorrectKey() {
		assertTrue(handler.isResponsible(card(CardHandlerKey.WITCH_BREW)));
		assertFalse(handler.isResponsible(card(CardHandlerKey.DISTRACT)));
	}

	// rust: allows_player_default_returns_true
	@Test
	public void allowsPlayerDefaultReturnsTrue() {
		assertTrue(handler.allowsPlayer(game, card(CardHandlerKey.WITCH_BREW), player));
	}

	// rust: activate_returns_true
	@Test
	public void activateReturnsTrue() {
		GameFixture.installScriptedDice(gameState, 2);
		assertTrue(handler.activate(card(CardHandlerKey.WITCH_BREW), step, player));
	}

	// rust: activate_roll_1_gives_mad_cap_mushroom_potion
	@Test
	public void activateRoll1GivesMadCapMushroomPotion() {
		GameFixture.installScriptedDice(gameState, 1);
		handler.activate(card(CardHandlerKey.WITCH_BREW), step, player);
		assertTrue(game.getFieldModel().hasCardEffect(player, CardEffect.MAD_CAP_MUSHROOM_POTION));
	}

	// rust: activate_roll_3_gives_sedative
	@Test
	public void activateRoll3GivesSedative() {
		GameFixture.installScriptedDice(gameState, 3);
		handler.activate(card(CardHandlerKey.WITCH_BREW), step, player);
		assertTrue(game.getFieldModel().hasCardEffect(player, CardEffect.SEDATIVE));
	}

	// rust: activate_roll_2_gives_no_effect
	@Test
	public void activateRoll2GivesNoEffect() {
		GameFixture.installScriptedDice(gameState, 2);
		handler.activate(card(CardHandlerKey.WITCH_BREW), step, player);
		assertFalse(game.getFieldModel().hasCardEffect(player, CardEffect.MAD_CAP_MUSHROOM_POTION));
		assertFalse(game.getFieldModel().hasCardEffect(player, CardEffect.SEDATIVE));
	}

	// rust: deactivate_removes_sedative
	@Test
	public void deactivateRemovesSedative() {
		game.getFieldModel().addCardEffect(player, CardEffect.SEDATIVE);
		assertTrue(game.getFieldModel().hasCardEffect(player, CardEffect.SEDATIVE));
		handler.deactivate(card(CardHandlerKey.WITCH_BREW), step, player);
		assertFalse(game.getFieldModel().hasCardEffect(player, CardEffect.SEDATIVE));
	}

	// rust: deactivate_removes_mad_cap_mushroom_potion
	@Test
	public void deactivateRemovesMadCapMushroomPotion() {
		game.getFieldModel().addCardEffect(player, CardEffect.MAD_CAP_MUSHROOM_POTION);
		handler.deactivate(card(CardHandlerKey.WITCH_BREW), step, player);
		assertFalse(game.getFieldModel().hasCardEffect(player, CardEffect.MAD_CAP_MUSHROOM_POTION));
	}

	// rust: deactivate_is_noop_when_no_effects
	@Test
	public void deactivateIsNoopWhenNoEffects() {
		handler.deactivate(card(CardHandlerKey.WITCH_BREW), step, player);
		assertFalse(game.getFieldModel().hasCardEffect(player, CardEffect.SEDATIVE));
	}

	// rust: get_name_returns_struct_name
	@Test
	public void getNameReturnsHandlerName() {
		assertEquals("WitchBrewHandler", handler.getName());
	}
}
