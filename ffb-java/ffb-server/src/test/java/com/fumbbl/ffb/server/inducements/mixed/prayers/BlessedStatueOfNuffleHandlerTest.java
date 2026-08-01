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
 * Mirror of ffb-rust
 * crates/ffb-engine/src/inducements/mixed/prayers/blessed_statue_of_nuffle_handler.rs tests
 * (portable subset — Rust headless init_effect grant tests are a documented divergence; Java
 * applies the effect via applySelection after the player-choice dialog).
 */
public class BlessedStatueOfNuffleHandlerTest {

	private GameState gameState;
	private Game game;
	private BlessedStatueOfNuffleHandler handler;

	@BeforeEach
	void setUp() {
		gameState = GameFixture.createGameState(3, RulesCollection.Rules.BB2020);
		game = gameState.getGame();
		handler = new com.fumbbl.ffb.server.inducements.bb2020.prayers.BlessedStatueOfNuffleHandler();
	}

	private boolean hasPro(RosterPlayer player) {
		return player.getSkillsIncludingTemporaryOnes().stream()
			.anyMatch(s -> "Pro".equals(s.getName()));
	}

	// rust: animation_type_is_correct
	@Test
	public void animationTypeIsCorrect() {
		assertEquals(AnimationType.PRAYER_BLESSED_STATUE_OF_NUFFLE, handler.animationType());
	}

	// rust: init_effect_returns_true (stub selector = no eligible players -> prayer wasted)
	@Test
	public void initEffectReturnsTrueWhenNoEligiblePlayers() {
		gameState = GameFixture.createGameState(0, RulesCollection.Rules.BB2020);
		game = gameState.getGame();
		assertTrue(handler.initEffect(gameState, game.getTeamHome()));
	}

	// rust: apply_selection_marks_and_grants_pro
	@Test
	public void applySelectionMarksAndGrantsPro() {
		RosterPlayer player = (RosterPlayer) game.getPlayerById("home1");
		assertFalse(hasPro(player));
		handler.applySelection(game, new PrayerDialogSelection("home1", null));
		assertTrue(player.hasActiveEnhancement(Prayer.BLESSED_STATUE_OF_NUFFLE.getName()));
		assertTrue(hasPro(player));
	}

	// rust: remove_effect_clears_enhancement
	@Test
	public void removeEffectClearsEnhancement() {
		RosterPlayer player = (RosterPlayer) game.getPlayerById("home1");
		game.getFieldModel().addPrayerEnhancements(player, Prayer.BLESSED_STATUE_OF_NUFFLE);
		assertTrue(player.hasActiveEnhancement(Prayer.BLESSED_STATUE_OF_NUFFLE.getName()));
		handler.removeEffectInternal(gameState, game.getTeamHome());
		assertFalse(player.hasActiveEnhancement(Prayer.BLESSED_STATUE_OF_NUFFLE.getName()));
	}
}
