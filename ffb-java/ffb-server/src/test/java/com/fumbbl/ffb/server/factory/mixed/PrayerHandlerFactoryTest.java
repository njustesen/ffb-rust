package com.fumbbl.ffb.server.factory.mixed;

import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.model.Game;
import com.fumbbl.ffb.server.GameState;
import com.fumbbl.ffb.server.fixture.GameFixture;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertNull;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirror of ffb-rust crates/ffb-engine/src/inducements/prayer_handler_factory.rs tests. The
 * Scanner-populated handler set is looked up by forName (case-insensitive by simple class name) and
 * forPrayer (handler.handles == identity match on handledPrayer()). Because handles() compares Prayer
 * instances by identity and BB2020/BB2025 have distinct Prayer enums, each edition's factory only
 * matches that edition's Prayer constants — so BB2020 has NECESSARY_VIOLENCE (no DAZZLING_CATCHING)
 * and BB2025 has DAZZLING_CATCHING (no NECESSARY_VIOLENCE).
 *
 * Exempt: initialize_bb20xx_registers_sixteen_handlers ×2 (Rust Vec len — no Java getHandlers/size
 * getter); new_factory_is_empty (folded into the forName/forPrayer before-init checks);
 * deactivate_prayers ×2 — Java's deactivatePrayers is a PRIVATE method on StepEndTurn (bb2020/bb2025),
 * not the factory (a Rust method-placement divergence); that behaviour is covered by StepEndTurn.
 */
public class PrayerHandlerFactoryTest {

	private PrayerHandlerFactory factoryFor(RulesCollection.Rules rules) {
		GameState gameState = GameFixture.createGameState(3, rules);
		Game game = gameState.getGame();
		PrayerHandlerFactory factory = new PrayerHandlerFactory();
		factory.initialize(game);
		return factory;
	}

	// rust: for_name_miss_returns_none (also covers new_factory_is_empty — no handlers before init)
	@Test
	public void forNameMissReturnsNull() {
		assertNull(new PrayerHandlerFactory().forName("Unknown"));
	}

	// rust: for_prayer_miss_returns_none (no handlers registered before initialize)
	@Test
	public void forPrayerMissBeforeInit() {
		assertFalse(new PrayerHandlerFactory()
			.forPrayer(com.fumbbl.ffb.inducement.bb2020.Prayer.FOULING_FRENZY).isPresent());
	}

	// rust: for_prayer_finds_fouling_frenzy_after_init_bb2020
	@Test
	public void forPrayerFindsFoulingFrenzyAfterInitBb2020() {
		assertTrue(factoryFor(RulesCollection.Rules.BB2020)
			.forPrayer(com.fumbbl.ffb.inducement.bb2020.Prayer.FOULING_FRENZY).isPresent());
	}

	// rust: for_prayer_finds_treacherous_trapdoor_after_init_bb2025
	@Test
	public void forPrayerFindsTreacherousTrapdoorAfterInitBb2025() {
		assertTrue(factoryFor(RulesCollection.Rules.BB2025)
			.forPrayer(com.fumbbl.ffb.inducement.bb2025.Prayer.TREACHEROUS_TRAPDOOR).isPresent());
	}

	// rust: for_prayer_finds_dazzling_catching_after_init_bb2025
	@Test
	public void forPrayerFindsDazzlingCatchingAfterInitBb2025() {
		assertTrue(factoryFor(RulesCollection.Rules.BB2025)
			.forPrayer(com.fumbbl.ffb.inducement.bb2025.Prayer.DAZZLING_CATCHING).isPresent());
	}

	// rust: bb2020_has_necessary_violence_bb2025_has_dazzling_catching
	@Test
	public void bb2020HasNecessaryViolenceBb2025HasDazzlingCatching() {
		PrayerHandlerFactory f2020 = factoryFor(RulesCollection.Rules.BB2020);
		assertTrue(f2020.forPrayer(com.fumbbl.ffb.inducement.bb2020.Prayer.NECESSARY_VIOLENCE).isPresent());
		assertFalse(f2020.forPrayer(com.fumbbl.ffb.inducement.bb2025.Prayer.DAZZLING_CATCHING).isPresent());

		PrayerHandlerFactory f2025 = factoryFor(RulesCollection.Rules.BB2025);
		assertTrue(f2025.forPrayer(com.fumbbl.ffb.inducement.bb2025.Prayer.DAZZLING_CATCHING).isPresent());
		assertFalse(f2025.forPrayer(com.fumbbl.ffb.inducement.bb2020.Prayer.NECESSARY_VIOLENCE).isPresent());
	}
}
