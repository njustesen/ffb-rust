package com.fumbbl.ffb.server.inducements.mixed.prayers;

import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.inducement.bb2020.Prayer;
import com.fumbbl.ffb.model.AnimationType;
import com.fumbbl.ffb.model.Game;
import com.fumbbl.ffb.model.RosterPlayer;
import com.fumbbl.ffb.server.GameState;
import com.fumbbl.ffb.server.fixture.GameFixture;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirror of ffb-rust crates/ffb-engine/src/inducements/mixed/prayers/iron_man_handler.rs tests
 * (portable subset), exercised through the concrete bb2020 subclass typed as the mixed class.
 * The Rust init_effect random-selection tests are a documented headless divergence: Java's
 * DialogPrayerHandler shows a player-choice dialog instead and applies the effect later via
 * applySelection.
 */
public class IronManHandlerTest {

	private GameState gameState;
	private Game game;
	private IronManHandler handler;

	@BeforeEach
	void setUp() {
		gameState = GameFixture.createGameState(3, RulesCollection.Rules.BB2020);
		game = gameState.getGame();
		handler = new com.fumbbl.ffb.server.inducements.bb2020.prayers.IronManHandler();
	}

	// rust: animation_type_is_correct
	@Test
	public void animationTypeIsCorrect() {
		assertEquals(AnimationType.PRAYER_IRON_MAN, handler.animationType());
	}

	// rust: init_effect_returns_true (stub selector = no eligible players -> prayer wasted)
	@Test
	public void initEffectReturnsTrueWhenNoEligiblePlayers() {
		gameState = GameFixture.createGameState(0, RulesCollection.Rules.BB2020);
		game = gameState.getGame();
		assertTrue(handler.initEffect(gameState, game.getTeamHome()));
	}

	// rust: apply_selection_marks_and_boosts_player
	@Test
	public void applySelectionMarksAndBoostsPlayer() {
		RosterPlayer player = (RosterPlayer) game.getPlayerById("home1");
		int baseArmour = player.getArmourWithModifiers();
		handler.applySelection(game, new PrayerDialogSelection("home1", null));
		assertTrue(player.hasActiveEnhancement(Prayer.IRON_MAN.getName()));
		assertEquals(baseArmour + 1, player.getArmourWithModifiers());
	}

	// rust: remove_effect_clears_enhancement
	@Test
	public void removeEffectClearsEnhancement() {
		RosterPlayer player = (RosterPlayer) game.getPlayerById("home1");
		game.getFieldModel().addPrayerEnhancements(player, Prayer.IRON_MAN);
		assertTrue(player.hasActiveEnhancement(Prayer.IRON_MAN.getName()));
		handler.removeEffectInternal(gameState, game.getTeamHome());
		assertFalse(player.hasActiveEnhancement(Prayer.IRON_MAN.getName()));
	}
}
