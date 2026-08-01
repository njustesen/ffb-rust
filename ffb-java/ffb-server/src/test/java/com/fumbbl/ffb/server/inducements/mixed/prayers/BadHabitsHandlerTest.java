package com.fumbbl.ffb.server.inducements.mixed.prayers;

import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.inducement.bb2020.Prayer;
import com.fumbbl.ffb.model.AnimationType;
import com.fumbbl.ffb.model.Game;
import com.fumbbl.ffb.model.Player;
import com.fumbbl.ffb.server.GameState;
import com.fumbbl.ffb.server.fixture.GameFixture;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirror of ffb-rust crates/ffb-engine/src/inducements/mixed/prayers/bad_habits_handler.rs tests.
 * The abstract mixed handler is exercised through the concrete bb2020 subclass, typed as the
 * mixed class so this same-package test can reach the protected affectedPlayers method.
 */
public class BadHabitsHandlerTest {

	private GameState gameState;
	private Game game;
	private BadHabitsHandler handler;

	@BeforeEach
	void setUp() {
		gameState = GameFixture.createGameState(3, RulesCollection.Rules.BB2020);
		game = gameState.getGame();
		handler = new com.fumbbl.ffb.server.inducements.bb2020.prayers.BadHabitsHandler();
	}

	// rust: animation_type_is_correct
	@Test
	public void animationTypeIsCorrect() {
		assertEquals(AnimationType.PRAYER_BAD_HABITS, handler.animationType());
	}

	// rust: affected_players_returns_d3_roll
	@Test
	public void affectedPlayersReturnsD3Roll() {
		GameFixture.installScriptedDice(gameState, 2, 3);
		assertEquals(2, handler.affectedPlayers(gameState));
		assertEquals(3, handler.affectedPlayers(gameState));
	}

	// rust: affected_players_returns_one_for_roll_one
	@Test
	public void affectedPlayersReturnsOneForRollOne() {
		GameFixture.installScriptedDice(gameState, 1);
		assertEquals(1, handler.affectedPlayers(gameState));
	}

	// rust: init_effect_returns_true
	@Test
	public void initEffectReturnsTrue() {
		GameFixture.installScriptedDice(gameState, 1);
		assertTrue(handler.initEffect(gameState, game.getTeamHome()));
	}

	// rust: remove_effect_is_callable
	@Test
	public void removeEffectIsCallable() {
		handler.removeEffectInternal(gameState, game.getTeamHome());
	}

	// rust: remove_effect_clears_enhancement_from_opponent_team (BAD_HABITS uses an
	// opponent-team selector, so removal must clear the enhancement from the OPPONENT roster)
	@Test
	public void removeEffectClearsEnhancementFromOpponentTeam() {
		Player<?> awayPlayer = game.getPlayerById("away1");
		game.getFieldModel().addPrayerEnhancements(awayPlayer, Prayer.BAD_HABITS);
		assertTrue(awayPlayer.hasActiveEnhancement(Prayer.BAD_HABITS.getName()));
		handler.removeEffectInternal(gameState, game.getTeamHome());
		assertFalse(awayPlayer.hasActiveEnhancement(Prayer.BAD_HABITS.getName()));
	}
}
