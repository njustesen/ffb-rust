package com.fumbbl.ffb.server.step.bb2020;

import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.model.Game;
import com.fumbbl.ffb.server.GameState;
import com.fumbbl.ffb.server.fixture.GameFixture;
import com.fumbbl.ffb.server.step.IStep;
import com.fumbbl.ffb.server.step.StepAction;
import com.fumbbl.ffb.server.step.StepId;
import com.fumbbl.ffb.server.step.StepParameter;
import com.fumbbl.ffb.server.step.StepParameterKey;
import com.fumbbl.ffb.server.step.StepParameterSet;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirror of ffb-rust crates/ffb-engine/src/step/bb2020/step_set_acting_player_and_team.rs. start()
 * sets the acting player to PLAYER_ID and flips homePlaying when that player's team differs from the
 * current acting team; then NEXT_STEP. PLAYER_ID is an init param. Exempt: no_player_id_returns_next
 * (Rust-defensive — Java derefs player.getTeam() and NPEs on a null id); set_parameter_player_id_accepted
 * (PLAYER_ID is init-consumed → setParameter returns false).
 */
public class StepSetActingPlayerAndTeamFixtureTest {

	private GameState gameState;
	private Game game;

	@BeforeEach
	public void setUp() {
		gameState = GameFixture.createGameState(3, RulesCollection.Rules.BB2020);
		game = gameState.getGame();
		GameFixture.placePlayer(gameState, "home1", 5, 5);
		GameFixture.placePlayer(gameState, "away1", 8, 8);
	}

	private IStep newStep(String playerId) {
		IStep step = GameFixture.createStep(gameState, StepId.SET_ACTING_PLAYER_AND_TEAM);
		StepParameterSet set = new StepParameterSet();
		set.add(StepParameter.from(StepParameterKey.PLAYER_ID, playerId));
		step.init(set);
		return step;
	}

	// rust: sets_acting_player_id
	@Test
	public void setsActingPlayerId() {
		GameFixture.startStep(newStep("home1"));
		assertEquals("home1", game.getActingPlayer().getPlayer().getId());
	}

	// rust: home_player_when_away_playing_toggles_home_playing
	@Test
	public void homePlayerWhenAwayPlayingTogglesHomePlaying() {
		game.setHomePlaying(false);
		GameFixture.startStep(newStep("home1"));
		assertTrue(game.isHomePlaying());
	}

	// rust: home_player_when_home_playing_no_toggle
	@Test
	public void homePlayerWhenHomePlayingNoToggle() {
		game.setHomePlaying(true);
		GameFixture.startStep(newStep("home1"));
		assertTrue(game.isHomePlaying());
	}

	// rust: away_player_when_home_playing_toggles
	@Test
	public void awayPlayerWhenHomePlayingToggles() {
		game.setHomePlaying(true);
		GameFixture.startStep(newStep("away1"));
		assertFalse(game.isHomePlaying());
	}

	// rust: returns_next_step_action
	@Test
	public void returnsNextStepAction() {
		assertEquals(StepAction.NEXT_STEP, GameFixture.startStep(newStep("home1")));
	}

	// rust: set_parameter_unknown_returns_false
	@Test
	public void setParameterUnknownReturnsFalse() {
		assertFalse(newStep("home1").setParameter(StepParameter.from(StepParameterKey.END_TURN, true)));
	}
}
