package com.fumbbl.ffb.server.step.bb2020;

import com.fumbbl.ffb.PlayerAction;
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
import static org.junit.jupiter.api.Assertions.assertNull;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirror of ffb-rust crates/ffb-engine/src/step/bb2020/step_set_acting_team.rs. start() clears the
 * acting player and, when the target team differs from the current acting team, flips homePlaying;
 * then NEXT_STEP. TEAM_ID is an init param (Rust threads it via setParameter — structural). The Rust
 * set_parameter_team_id_accepted twin is EXEMPT: TEAM_ID is init-consumed in Java, so setParameter
 * returns false for it.
 */
public class StepSetActingTeamFixtureTest {

	private GameState gameState;
	private Game game;

	@BeforeEach
	public void setUp() {
		gameState = GameFixture.createGameState(3, RulesCollection.Rules.BB2020);
		game = gameState.getGame();
		GameFixture.placePlayer(gameState, "home1", 5, 5);
		GameFixture.setActingPlayer(gameState, "home1", PlayerAction.MOVE);
	}

	private IStep newStep(String teamId) {
		IStep step = GameFixture.createStep(gameState, StepId.SET_ACTING_TEAM);
		StepParameterSet set = new StepParameterSet();
		if (teamId != null) {
			set.add(StepParameter.from(StepParameterKey.TEAM_ID, teamId));
		}
		step.init(set);
		return step;
	}

	private String homeId() { return game.getTeamHome().getId(); }
	private String awayId() { return game.getTeamAway().getId(); }

	// rust: clears_acting_player
	@Test
	public void clearsActingPlayer() {
		GameFixture.startStep(newStep(homeId()));
		assertNull(game.getActingPlayer().getPlayer());
	}

	// rust: no_team_id_clears_player_and_returns_next
	@Test
	public void noTeamIdClearsPlayerAndReturnsNext() {
		assertEquals(StepAction.NEXT_STEP, GameFixture.startStep(newStep(null)));
		assertNull(game.getActingPlayer().getPlayer());
	}

	// rust: home_team_when_away_playing_toggles
	@Test
	public void homeTeamWhenAwayPlayingToggles() {
		game.setHomePlaying(false);
		GameFixture.startStep(newStep(homeId()));
		assertTrue(game.isHomePlaying());
	}

	// rust: home_team_when_home_playing_no_toggle
	@Test
	public void homeTeamWhenHomePlayingNoToggle() {
		game.setHomePlaying(true);
		GameFixture.startStep(newStep(homeId()));
		assertTrue(game.isHomePlaying());
	}

	// rust: away_team_when_home_playing_toggles
	@Test
	public void awayTeamWhenHomePlayingToggles() {
		game.setHomePlaying(true);
		GameFixture.startStep(newStep(awayId()));
		assertFalse(game.isHomePlaying());
	}

	// rust: returns_next_step_action
	@Test
	public void returnsNextStepAction() {
		assertEquals(StepAction.NEXT_STEP, GameFixture.startStep(newStep(homeId())));
	}

	// rust: set_parameter_unknown_returns_false
	@Test
	public void setParameterUnknownReturnsFalse() {
		assertFalse(newStep(homeId()).setParameter(StepParameter.from(StepParameterKey.END_TURN, true)));
	}
}
