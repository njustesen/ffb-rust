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
 * Mirror of ffb-rust crates/ffb-engine/src/inducements/mixed/prayers/knuckle_dusters_handler.rs
 * tests (portable subset), exercised through the concrete bb2020 subclass typed as the mixed
 * class. The Rust init_effect random-selection test is a documented headless divergence: Java's
 * DialogPrayerHandler shows a player-choice dialog instead and applies the effect later via
 * applySelection.
 */
public class KnuckleDustersHandlerTest {

	private GameState gameState;
	private Game game;
	private KnuckleDustersHandler handler;

	@BeforeEach
	void setUp() {
		gameState = GameFixture.createGameState(3, RulesCollection.Rules.BB2020);
		game = gameState.getGame();
		handler = new com.fumbbl.ffb.server.inducements.bb2020.prayers.KnuckleDustersHandler();
	}

	private boolean hasMightyBlow(RosterPlayer player) {
		return player.getSkillsIncludingTemporaryOnes().stream()
			.anyMatch(s -> "Mighty Blow".equals(s.getName()));
	}

	// rust: animation_type_is_correct
	@Test
	public void animationTypeIsCorrect() {
		assertEquals(AnimationType.PRAYER_KNUCKLE_DUSTERS, handler.animationType());
	}

	// rust: init_effect_returns_true (stub selector = no eligible players -> prayer wasted)
	@Test
	public void initEffectReturnsTrueWhenNoEligiblePlayers() {
		gameState = GameFixture.createGameState(0, RulesCollection.Rules.BB2020);
		game = gameState.getGame();
		assertTrue(handler.initEffect(gameState, game.getTeamHome()));
	}

	// rust: apply_selection_marks_and_grants_mighty_blow
	@Test
	public void applySelectionMarksAndGrantsMightyBlow() {
		RosterPlayer player = (RosterPlayer) game.getPlayerById("home1");
		assertFalse(hasMightyBlow(player));
		handler.applySelection(game, new PrayerDialogSelection("home1", null));
		assertTrue(player.hasActiveEnhancement(Prayer.KNUCKLE_DUSTERS.getName()));
		assertTrue(hasMightyBlow(player));
	}

	// rust: remove_effect_clears_enhancement
	@Test
	public void removeEffectClearsEnhancement() {
		RosterPlayer player = (RosterPlayer) game.getPlayerById("home1");
		game.getFieldModel().addPrayerEnhancements(player, Prayer.KNUCKLE_DUSTERS);
		assertTrue(player.hasActiveEnhancement(Prayer.KNUCKLE_DUSTERS.getName()));
		handler.removeEffectInternal(gameState, game.getTeamHome());
		assertFalse(player.hasActiveEnhancement(Prayer.KNUCKLE_DUSTERS.getName()));
	}
}
