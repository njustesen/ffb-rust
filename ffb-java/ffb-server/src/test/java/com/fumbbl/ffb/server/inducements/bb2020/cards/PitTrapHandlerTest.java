package com.fumbbl.ffb.server.inducements.bb2020.cards;

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
 * Mirror of ffb-rust crates/ffb-engine/src/inducements/bb2020/cards/pit_trap_handler.rs tests.
 */
public class PitTrapHandlerTest {

	private GameState gameState;
	private Game game;
	private IStep step;
	private PitTrapHandler handler;

	@BeforeEach
	void setUp() {
		gameState = GameFixture.createGameState(3, RulesCollection.Rules.BB2020);
		game = gameState.getGame();
		step = GameFixture.createStep(gameState, StepId.INIT_START_GAME);
		handler = new PitTrapHandler();
	}

	private Card card(CardHandlerKey key) {
		return new Card("Test Card", "tc", com.fumbbl.ffb.inducement.bb2020.CardType.DIRTY_TRICK,
			CardTarget.ANY_PLAYER, false, new InducementPhase[0], InducementDuration.UNTIL_END_OF_GAME,
			"test", key);
	}

	// rust: is_responsible_for_correct_key
	@Test
	public void isResponsibleForCorrectKey() {
		assertTrue(handler.isResponsible(card(CardHandlerKey.PIT_TRAP)));
		assertFalse(handler.isResponsible(card(CardHandlerKey.RABBITS_FOOT)));
	}

	// rust: allows_player_default_returns_true
	@Test
	public void allowsPlayerDefaultReturnsTrue() {
		assertTrue(handler.allowsPlayer(game, card(CardHandlerKey.PIT_TRAP), game.getPlayerById("home1")));
	}

	// rust: activate_on_game_returns_true
	@Test
	public void activateReturnsTrue() {
		GameFixture.placePlayer(gameState, "home1", 5, 5);
		assertTrue(handler.activate(card(CardHandlerKey.PIT_TRAP), step, game.getPlayerById("home1")));
	}

	// rust: activation_parameters_drops_the_player_prone
	@Test
	public void activateDropsThePlayerProne() {
		GameFixture.placePlayer(gameState, "home1", 5, 5);
		Player<?> player = game.getPlayerById("home1");
		handler.activate(card(CardHandlerKey.PIT_TRAP), step, player);
		assertEquals(PlayerState.PRONE, game.getFieldModel().getPlayerState(player).getBase());
	}

	// rust: activation_parameters_scatters_the_ball_when_player_is_carrying_it
	@Test
	public void activateScattersBallWhenPlayerIsCarryingIt() {
		GameFixture.placePlayer(gameState, "home1", 5, 5);
		Player<?> player = game.getPlayerById("home1");
		game.getFieldModel().setBallCoordinate(game.getFieldModel().getPlayerCoordinate(player));
		game.getFieldModel().setBallMoving(false);
		GameFixture.installScriptedDice(gameState, 1, 1, 1, 1);
		handler.activate(card(CardHandlerKey.PIT_TRAP), step, player);
		assertTrue(game.getFieldModel().isBallMoving());
	}

	// rust: get_name_returns_struct_name
	@Test
	public void getNameReturnsHandlerName() {
		assertEquals("PitTrapHandler", handler.getName());
	}
}
