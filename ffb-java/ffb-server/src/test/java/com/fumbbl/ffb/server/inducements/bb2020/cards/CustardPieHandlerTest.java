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
 * Mirror of ffb-rust crates/ffb-engine/src/inducements/bb2020/cards/custard_pie_handler.rs tests.
 */
public class CustardPieHandlerTest {

	private GameState gameState;
	private Game game;
	private IStep step;
	private CustardPieHandler handler;

	@BeforeEach
	void setUp() {
		gameState = GameFixture.createGameState(3, RulesCollection.Rules.BB2020);
		game = gameState.getGame();
		step = GameFixture.createStep(gameState, StepId.INIT_START_GAME);
		handler = new CustardPieHandler();
	}

	private Card card(CardHandlerKey key) {
		return new Card("Test Card", "tc", com.fumbbl.ffb.inducement.bb2020.CardType.DIRTY_TRICK,
			CardTarget.ANY_PLAYER, false, new InducementPhase[0], InducementDuration.UNTIL_END_OF_GAME,
			"test", key);
	}

	// rust: is_responsible_for_correct_key
	@Test
	public void isResponsibleForCorrectKey() {
		assertTrue(handler.isResponsible(card(CardHandlerKey.CUSTARD_PIE)));
		assertFalse(handler.isResponsible(card(CardHandlerKey.RABBITS_FOOT)));
	}

	// rust: allows_player_true_for_adjacent_prone_teammate (raw card is in neither inducement
	// set, so ownTeam resolves to away — check an away player with an adjacent away teammate)
	@Test
	public void allowsPlayerTrueForAdjacentTeammate() {
		GameFixture.placePlayer(gameState, "away1", 5, 5);
		GameFixture.placePlayer(gameState, "away2", 5, 6);
		assertTrue(handler.allowsPlayer(game, card(CardHandlerKey.CUSTARD_PIE), game.getPlayerById("away1")));
	}

	// rust: allows_player_false_for_unknown_player (Java variant: player off-pitch has no
	// adjacent teammates)
	@Test
	public void allowsPlayerFalseForIsolatedPlayer() {
		GameFixture.placePlayer(gameState, "away1", 5, 5);
		assertFalse(handler.allowsPlayer(game, card(CardHandlerKey.CUSTARD_PIE), game.getPlayerById("away1")));
	}

	// rust: activate_sets_hypnotized_state
	@Test
	public void activateSetsHypnotizedState() {
		GameFixture.placePlayer(gameState, "home1", 5, 5);
		Player<?> player = game.getPlayerById("home1");
		handler.activate(card(CardHandlerKey.CUSTARD_PIE), step, player);
		assertTrue(game.getFieldModel().getPlayerState(player).isHypnotized());
	}

	// rust: deactivate_clears_hypnotized_state
	@Test
	public void deactivateClearsHypnotizedState() {
		GameFixture.placePlayer(gameState, "home1", 5, 5);
		Player<?> player = game.getPlayerById("home1");
		PlayerState state = game.getFieldModel().getPlayerState(player);
		game.getFieldModel().setPlayerState(player, state.changeHypnotized(true));
		handler.deactivate(card(CardHandlerKey.CUSTARD_PIE), step, player);
		assertFalse(game.getFieldModel().getPlayerState(player).isHypnotized());
	}

	// rust: get_name_returns_struct_name
	@Test
	public void getNameReturnsHandlerName() {
		assertEquals("CustardPieHandler", handler.getName());
	}
}
