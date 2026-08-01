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
 * Mirror of ffb-rust
 * crates/ffb-engine/src/inducements/mixed/prayers/moles_under_the_pitch_handler.rs tests,
 * exercised through the concrete bb2020 subclass typed as the mixed class.
 */
public class MolesUnderThePitchHandlerTest {

	private GameState gameState;
	private Game game;
	private MolesUnderThePitchHandler handler;

	@BeforeEach
	void setUp() {
		gameState = GameFixture.createGameState(3, RulesCollection.Rules.BB2020);
		game = gameState.getGame();
		handler = new com.fumbbl.ffb.server.inducements.bb2020.prayers.MolesUnderThePitchHandler();
	}

	private boolean hasMoles(String teamId) {
		return gameState.getPrayerState().getMolesUnderThePitch().contains(teamId);
	}

	// rust: init_effect_adds_moles_under_the_pitch
	@Test
	public void initEffectAddsMolesUnderThePitch() {
		assertTrue(handler.initEffect(gameState, game.getTeamHome()));
		assertTrue(hasMoles(game.getTeamHome().getId()));
	}

	// rust: remove_effect_removes_moles
	@Test
	public void removeEffectRemovesMoles() {
		gameState.getPrayerState().addMolesUnderThePitch(game.getTeamHome());
		handler.removeEffectInternal(gameState, game.getTeamHome());
		assertFalse(hasMoles(game.getTeamHome().getId()));
	}

	// rust: animation_type_is_correct
	@Test
	public void animationTypeIsCorrect() {
		assertEquals(AnimationType.PRAYER_MOLES_UNDER_THE_PITCH, handler.animationType());
	}

	// rust: init_effect_does_not_affect_other_team
	@Test
	public void initEffectDoesNotAffectOtherTeam() {
		handler.initEffect(gameState, game.getTeamHome());
		assertFalse(hasMoles(game.getTeamAway().getId()));
	}

	// rust: remove_effect_on_missing_team_is_safe
	@Test
	public void removeEffectOnMissingTeamIsSafe() {
		handler.removeEffectInternal(gameState, game.getTeamAway());
		assertFalse(hasMoles(game.getTeamAway().getId()));
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
		assertFalse(hasMoles(game.getTeamHome().getId()));
	}
}
