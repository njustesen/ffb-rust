package com.fumbbl.ffb.server.inducements.mixed.prayers;

import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.inducement.bb2020.Prayer;
import com.fumbbl.ffb.model.Game;
import com.fumbbl.ffb.model.RosterPlayer;
import com.fumbbl.ffb.server.GameState;
import com.fumbbl.ffb.server.fixture.GameFixture;
import com.fumbbl.ffb.server.inducements.bb2020.prayers.OpponentPlayerSelector;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirror of ffb-rust crates/ffb-engine/src/inducements/mixed/prayers/enhancement_remover.rs
 * tests (portable subset).
 */
public class EnhancementRemoverTest {

	private GameState gameState;
	private Game game;
	private EnhancementRemover remover;

	@BeforeEach
	void setUp() {
		gameState = GameFixture.createGameState(3, RulesCollection.Rules.BB2020);
		game = gameState.getGame();
		remover = new EnhancementRemover();
	}

	private RosterPlayer player(String id) {
		return (RosterPlayer) game.getPlayerById(id);
	}

	private boolean enhanced(String id, Prayer prayer) {
		return player(id).hasActiveEnhancement(prayer.getName());
	}

	// rust: remove_enhancement_is_callable
	@Test
	public void removeEnhancementIsCallable() {
		remover.removeEnhancement(gameState, game.getTeamHome(),
			com.fumbbl.ffb.server.inducements.bb2020.prayers.PlayerSelector.INSTANCE, Prayer.STILETTO);
	}

	// rust: remove_enhancement_clears_tracking
	@Test
	public void removeEnhancementClearsTracking() {
		game.getFieldModel().addPrayerEnhancements(player("home1"), Prayer.STILETTO);
		assertTrue(enhanced("home1", Prayer.STILETTO));
		remover.removeEnhancement(gameState, game.getTeamHome(),
			com.fumbbl.ffb.server.inducements.bb2020.prayers.PlayerSelector.INSTANCE, Prayer.STILETTO);
		assertFalse(enhanced("home1", Prayer.STILETTO));
	}

	// rust: remove_enhancement_only_affects_own_team
	@Test
	public void removeEnhancementOnlyAffectsOwnTeam() {
		game.getFieldModel().addPrayerEnhancements(player("home1"), Prayer.STILETTO);
		game.getFieldModel().addPrayerEnhancements(player("away1"), Prayer.STILETTO);
		remover.removeEnhancement(gameState, game.getTeamHome(),
			com.fumbbl.ffb.server.inducements.bb2020.prayers.PlayerSelector.INSTANCE, Prayer.STILETTO);
		assertFalse(enhanced("home1", Prayer.STILETTO));
		assertTrue(enhanced("away1", Prayer.STILETTO));
	}

	// rust: remove_enhancement_targets_opponent_team_when_selector_resolves_to_opponent
	@Test
	public void removeEnhancementTargetsOpponentTeamWhenSelectorResolvesToOpponent() {
		game.getFieldModel().addPrayerEnhancements(player("away1"), Prayer.BAD_HABITS);
		remover.removeEnhancement(gameState, game.getTeamHome(),
			OpponentPlayerSelector.INSTANCE, Prayer.BAD_HABITS);
		assertFalse(enhanced("away1", Prayer.BAD_HABITS));
		assertFalse(enhanced("home1", Prayer.BAD_HABITS));
	}
}
