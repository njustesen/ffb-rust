package com.fumbbl.ffb.server.util;

import com.fumbbl.ffb.CardTarget;
import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.inducement.Card;
import com.fumbbl.ffb.inducement.CardHandlerKey;
import com.fumbbl.ffb.inducement.InducementDuration;
import com.fumbbl.ffb.inducement.InducementPhase;
import com.fumbbl.ffb.model.Game;
import com.fumbbl.ffb.model.Player;
import com.fumbbl.ffb.server.GameState;
import com.fumbbl.ffb.server.fixture.GameFixture;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import java.util.Arrays;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirror of ffb-rust crates/ffb-engine/src/util/util_server_cards.rs findAllowedPlayersForCard
 * tests. Non-player-targeted cards allow nobody; OWN/OPPOSING filter by the card-owning team
 * (owner = whichever turn-data inducement set the card is available in).
 */
public class UtilServerCardsTest {

	private GameState gameState;
	private Game game;

	@BeforeEach
	void setUp() {
		gameState = GameFixture.createGameState(3, RulesCollection.Rules.BB2025);
		game = gameState.getGame();
	}

	private Card card(CardTarget target, CardHandlerKey key) {
		return new Card("Test Card", "tc", com.fumbbl.ffb.inducement.bb2020.CardType.DIRTY_TRICK,
			target, false, new InducementPhase[0], InducementDuration.UNTIL_END_OF_GAME, "test", key);
	}

	// rust: find_allowed_players_for_card_excludes_non_player_targeted_cards (TURN target)
	@Test
	public void excludesNonPlayerTargetedCards() {
		assertEquals(0, UtilServerCards.findAllowedPlayersForCard(game, card(CardTarget.TURN, null)).length);
	}

	// rust: find_allowed_players_for_card_filters_by_own_player_target
	@Test
	public void filtersByOwnPlayerTarget() {
		Card c = card(CardTarget.OWN_PLAYER, null);
		game.getTurnDataHome().getInducementSet().addAvailableCard(c);
		Player<?>[] allowed = UtilServerCards.findAllowedPlayersForCard(game, c);
		assertTrue(allowed.length > 0);
		assertTrue(Arrays.stream(allowed).allMatch(p -> game.getTeamHome().hasPlayer(p)));
	}

	// rust: find_allowed_players_for_card_filters_by_opposing_player_target
	@Test
	public void filtersByOpposingPlayerTarget() {
		// no registered handler for this key, isolating the OPPOSING_PLAYER filter
		Card c = card(CardTarget.OPPOSING_PLAYER, null);
		game.getTurnDataHome().getInducementSet().addAvailableCard(c);
		Player<?>[] allowed = UtilServerCards.findAllowedPlayersForCard(game, c);
		assertTrue(allowed.length > 0);
		assertTrue(Arrays.stream(allowed).allMatch(p -> game.getTeamAway().hasPlayer(p)));
	}
}
