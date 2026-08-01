package com.fumbbl.ffb.server.inducements.mixed.prayers;

import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.model.AnimationType;
import com.fumbbl.ffb.model.Game;
import com.fumbbl.ffb.server.GameState;
import com.fumbbl.ffb.server.fixture.GameFixture;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirror of ffb-rust crates/ffb-engine/src/inducements/mixed/prayers/fan_interaction_handler.rs
 * tests, exercised through the concrete bb2020 subclass typed as the mixed class.
 */
public class FanInteractionHandlerTest {

	private GameState gameState;
	private Game game;
	private FanInteractionHandler handler;

	@BeforeEach
	void setUp() {
		gameState = GameFixture.createGameState(3, RulesCollection.Rules.BB2020);
		game = gameState.getGame();
		handler = new com.fumbbl.ffb.server.inducements.bb2020.prayers.FanInteractionHandler();
	}

	// rust: init_effect_adds_fan_interaction
	@Test
	public void initEffectAddsFanInteraction() {
		assertTrue(handler.initEffect(gameState, game.getTeamHome()));
		assertTrue(gameState.getPrayerState().hasFanInteraction(game.getTeamHome()));
	}

	// rust: remove_effect_removes_fan_interaction
	@Test
	public void removeEffectRemovesFanInteraction() {
		gameState.getPrayerState().addFanInteraction(game.getTeamHome());
		handler.removeEffectInternal(gameState, game.getTeamHome());
		assertFalse(gameState.getPrayerState().hasFanInteraction(game.getTeamHome()));
	}

	// rust: animation_type_is_correct
	@Test
	public void animationTypeIsCorrect() {
		assertEquals(AnimationType.PRAYER_FAN_INTERACTION, handler.animationType());
	}

	// rust: init_effect_does_not_affect_other_team
	@Test
	public void initEffectDoesNotAffectOtherTeam() {
		handler.initEffect(gameState, game.getTeamHome());
		assertFalse(gameState.getPrayerState().hasFanInteraction(game.getTeamAway()));
	}

	// rust: remove_effect_on_missing_team_is_safe
	@Test
	public void removeEffectOnMissingTeamIsSafe() {
		handler.removeEffectInternal(gameState, game.getTeamAway());
		assertFalse(gameState.getPrayerState().hasFanInteraction(game.getTeamAway()));
	}

	// rust: init_effect_returns_true_always
	@Test
	public void initEffectReturnsTrueAlways() {
		assertTrue(handler.initEffect(gameState, game.getTeamAway()));
	}

	// rust: double_add_and_remove_leaves_clean_state
	@Test
	public void doubleAddAndRemoveLeavesCleanState() {
		handler.initEffect(gameState, game.getTeamHome());
		handler.initEffect(gameState, game.getTeamHome());
		handler.removeEffectInternal(gameState, game.getTeamHome());
		assertFalse(gameState.getPrayerState().hasFanInteraction(game.getTeamHome()));
	}
}
