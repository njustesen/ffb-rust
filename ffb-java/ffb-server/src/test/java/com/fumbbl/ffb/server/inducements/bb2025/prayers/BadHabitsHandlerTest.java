package com.fumbbl.ffb.server.inducements.bb2025.prayers;

import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.inducement.bb2025.Prayer;
import com.fumbbl.ffb.model.AnimationType;
import com.fumbbl.ffb.model.Game;
import com.fumbbl.ffb.server.GameState;
import com.fumbbl.ffb.server.fixture.GameFixture;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import java.util.Arrays;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirror of ffb-rust crates/ffb-engine/src/inducements/bb2025/prayers/bad_habits_handler.rs
 * tests. The BB2025 variant additionally passes the Loner skill (hasToRollToUseTeamReroll) as an
 * added skill, and selects from the opponent's team.
 */
public class BadHabitsHandlerTest {

	private GameState gameState;
	private Game game;
	private BadHabitsHandler handler;

	@BeforeEach
	void setUp() {
		gameState = GameFixture.createGameState(3, RulesCollection.Rules.BB2025);
		game = gameState.getGame();
		handler = new BadHabitsHandler();
	}

	// rust: handles_prayer_bad_habits
	@Test
	public void handlesPrayerBadHabits() {
		assertEquals(Prayer.BAD_HABITS, handler.handledPrayer());
		assertTrue(handler.handles(Prayer.BAD_HABITS));
	}

	// rust: does_not_handle_other_prayers
	@Test
	public void doesNotHandleOtherPrayers() {
		assertFalse(handler.handles(Prayer.PERFECT_PASSING));
	}

	// rust: animation_type_is_correct
	@Test
	public void animationTypeIsCorrect() {
		assertEquals(AnimationType.PRAYER_BAD_HABITS, handler.animationType());
	}

	// rust: init_effect_returns_true
	@Test
	public void initEffectReturnsTrue() {
		GameFixture.installScriptedDice(gameState, 1);
		assertTrue(handler.initEffect(gameState, game.getTeamHome()));
	}

	// rust: init_effect_selects_from_opponent_team
	@Test
	public void initEffectSelectsFromOpponentTeam() {
		GameFixture.installScriptedDice(gameState, 3);
		handler.initEffect(gameState, game.getTeamHome());
		boolean anyAwayEnhanced = Arrays.stream(game.getTeamAway().getPlayers())
			.anyMatch(p -> p.hasActiveEnhancement(Prayer.BAD_HABITS.getName()));
		boolean anyHomeEnhanced = Arrays.stream(game.getTeamHome().getPlayers())
			.anyMatch(p -> p.hasActiveEnhancement(Prayer.BAD_HABITS.getName()));
		assertTrue(anyAwayEnhanced);
		assertFalse(anyHomeEnhanced);
	}
}
