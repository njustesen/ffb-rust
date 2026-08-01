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
 * Mirror of ffb-rust crates/ffb-engine/src/inducements/mixed/prayers/under_scrutiny_handler.rs
 * tests, exercised through the concrete bb2020 subclass typed as the mixed class. UNDER_SCRUTINY
 * targets the OPPONENT of the praying team.
 */
public class UnderScrutinyHandlerTest {

	private GameState gameState;
	private Game game;
	private UnderScrutinyHandler handler;

	@BeforeEach
	void setUp() {
		gameState = GameFixture.createGameState(3, RulesCollection.Rules.BB2020);
		game = gameState.getGame();
		handler = new com.fumbbl.ffb.server.inducements.bb2020.prayers.UnderScrutinyHandler();
	}

	// rust: init_effect_adds_scrutiny_to_opponent
	@Test
	public void initEffectAddsScrutinyToOpponent() {
		assertTrue(handler.initEffect(gameState, game.getTeamHome()));
		assertTrue(gameState.getPrayerState().isUnderScrutiny(game.getTeamAway()));
		assertFalse(gameState.getPrayerState().isUnderScrutiny(game.getTeamHome()));
	}

	// rust: remove_effect_removes_scrutiny_from_opponent
	@Test
	public void removeEffectRemovesScrutinyFromOpponent() {
		gameState.getPrayerState().addUnderScrutiny(game.getTeamAway());
		handler.removeEffectInternal(gameState, game.getTeamHome());
		assertFalse(gameState.getPrayerState().isUnderScrutiny(game.getTeamAway()));
	}

	// rust: animation_type_is_correct (its duplicate animation_type_is_under_scrutiny was pruned)
	@Test
	public void animationTypeIsCorrect() {
		assertEquals(AnimationType.PRAYER_UNDER_SCRUTINY, handler.animationType());
	}

	// rust: init_effect_does_not_add_scrutiny_to_praying_team
	@Test
	public void initEffectDoesNotAddScrutinyToPrayingTeam() {
		handler.initEffect(gameState, game.getTeamHome());
		assertFalse(gameState.getPrayerState().isUnderScrutiny(game.getTeamHome()));
	}

	// rust: remove_effect_on_missing_team_is_safe
	@Test
	public void removeEffectOnMissingTeamIsSafe() {
		handler.removeEffectInternal(gameState, game.getTeamHome());
		assertFalse(gameState.getPrayerState().isUnderScrutiny(game.getTeamAway()));
	}

	// rust: init_effect_away_team_adds_scrutiny_to_home
	@Test
	public void initEffectAwayTeamAddsScrutinyToHome() {
		assertTrue(handler.initEffect(gameState, game.getTeamAway()));
		assertTrue(gameState.getPrayerState().isUnderScrutiny(game.getTeamHome()));
		assertFalse(gameState.getPrayerState().isUnderScrutiny(game.getTeamAway()));
	}
}
