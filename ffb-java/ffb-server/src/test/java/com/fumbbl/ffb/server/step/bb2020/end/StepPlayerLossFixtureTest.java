package com.fumbbl.ffb.server.step.bb2020.end;

import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.model.Game;
import com.fumbbl.ffb.model.PlayerResult;
import com.fumbbl.ffb.server.GameState;
import com.fumbbl.ffb.server.fixture.GameFixture;
import com.fumbbl.ffb.server.step.IStep;
import com.fumbbl.ffb.server.step.StepAction;
import com.fumbbl.ffb.server.step.StepId;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;

/**
 * Mirror of ffb-rust crates/ffb-engine/src/step/.../step_player_loss.rs (concession-guard subset).
 * Player-loss defection only runs for a team that conceded ILLEGALLY; otherwise the step is a
 * NEXT_STEP no-op. The defection-eligibility (3+ advancements over position) + rollPlayerLoss +
 * ReportDefectingPlayers tests are dice/eligibility-driven and deferred.
 */
public class StepPlayerLossFixtureTest {

	private GameState gameState;
	private Game game;

	@BeforeEach
	public void setUp() {
		gameState = GameFixture.createGameState(3, RulesCollection.Rules.BB2020);
		game = gameState.getGame();
	}

	private IStep newStep() {
		return GameFixture.createStep(gameState, StepId.PLAYER_LOSS);
	}

	// rust: no_concession_returns_next
	@Test
	public void noConcessionReturnsNext() {
		assertEquals(StepAction.NEXT_STEP, GameFixture.startStep(newStep()));
	}

	// rust: legal_concession_skips_defection
	@Test
	public void legalConcessionSkipsDefection() {
		game.getGameResult().getTeamResultHome().setConceded(true);
		game.setConcededLegally(true);
		GameFixture.placePlayer(gameState, "home1", 5, 5);
		PlayerResult pr = game.getGameResult().getPlayerResult(game.getPlayerById("home1"));
		GameFixture.startStep(newStep());
		assertFalse(pr.isDefecting());
	}
}
