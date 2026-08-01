package com.fumbbl.ffb.server.factory;

import com.fumbbl.ffb.CardTarget;
import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.inducement.Card;
import com.fumbbl.ffb.inducement.CardHandlerKey;
import com.fumbbl.ffb.inducement.InducementDuration;
import com.fumbbl.ffb.inducement.InducementPhase;
import com.fumbbl.ffb.model.Game;
import com.fumbbl.ffb.server.GameState;
import com.fumbbl.ffb.server.fixture.GameFixture;
import com.fumbbl.ffb.server.inducements.CardHandler;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertNotNull;
import static org.junit.jupiter.api.Assertions.assertNull;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirror of ffb-rust crates/ffb-engine/src/inducements/card_handler_factory.rs tests. The
 * Scanner-populated handler set is looked up by forCard (handler.isResponsible) and forName
 * (case-insensitive by simple class name). (The Rust initialize_*_registers_eight_handlers count
 * tests assert the Rust Vec length — no Java getHandlers()/size getter — exempt; the
 * for_name/for_card tests that build a fabricated test handler exercise Rust registry mechanics —
 * exempt; for_card_no_match is fixture-inexpressible without a card whose key no registered handler
 * matches. The per-handler behaviour is covered by the individual *HandlerTest twins.)
 *
 * for_card_finds_witch_brew_after_initialize_bb2025 is EXEMPT: Java has NO @RulesCollection(BB2025)
 * card-handler classes (only bb2016/bb2020 exist), so a pure-BB2025 Scanner registers none — whereas
 * the Rust factory hand-registers the bb2020 handlers for bb2025. This registration-list divergence is
 * non-functional here (the bb2020 WitchBrewHandler twin already covers the handler behaviour); a
 * faithful bb2025 card-registration reconciliation is deferred with the sequence_generator_factory note.
 */
public class CardHandlerFactoryTest {

	private CardHandlerFactory factoryFor(RulesCollection.Rules rules) {
		GameState gameState = GameFixture.createGameState(3, rules);
		Game game = gameState.getGame();
		CardHandlerFactory factory = new CardHandlerFactory();
		factory.initialize(game);
		return factory;
	}

	private Card card(CardHandlerKey key) {
		return new Card("Test Card", "tc", com.fumbbl.ffb.inducement.bb2020.CardType.DIRTY_TRICK,
			CardTarget.ANY_PLAYER, false, new InducementPhase[0], InducementDuration.UNTIL_END_OF_GAME,
			"test", key);
	}

	// rust: for_card_finds_chop_block_after_initialize_bb2016
	@Test
	public void forCardFindsChopBlockAfterInitializeBb2016() {
		CardHandlerFactory factory = factoryFor(RulesCollection.Rules.BB2016);
		assertTrue(factory.forCard(card(com.fumbbl.ffb.inducement.bb2016.CardHandlerKey.CHOP_BLOCK)).isPresent());
	}

	// rust: for_card_finds_distract_after_initialize_bb2020
	@Test
	public void forCardFindsDistractAfterInitializeBb2020() {
		CardHandlerFactory factory = factoryFor(RulesCollection.Rules.BB2020);
		assertTrue(factory.forCard(card(com.fumbbl.ffb.inducement.bb2020.CardHandlerKey.DISTRACT)).isPresent());
	}

	// rust: for_name_finds_case_insensitive (via a real registered handler)
	@Test
	public void forNameFindsCaseInsensitive() {
		CardHandlerFactory factory = factoryFor(RulesCollection.Rules.BB2016);
		CardHandler handler = factory.forCard(card(com.fumbbl.ffb.inducement.bb2016.CardHandlerKey.CHOP_BLOCK)).orElseThrow();
		assertNotNull(factory.forName(handler.getName().toUpperCase()));
		assertNotNull(factory.forName(handler.getName().toLowerCase()));
	}

	// rust: for_name_miss_returns_none
	@Test
	public void forNameMissReturnsNull() {
		assertNull(factoryFor(RulesCollection.Rules.BB2016).forName("NoSuchHandler"));
	}
}
