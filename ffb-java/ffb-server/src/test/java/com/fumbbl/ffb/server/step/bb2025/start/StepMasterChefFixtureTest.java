package com.fumbbl.ffb.server.step.bb2025.start;

import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.model.Game;
import com.fumbbl.ffb.server.GameState;
import com.fumbbl.ffb.server.fixture.GameFixture;
import com.fumbbl.ffb.server.step.IStep;
import com.fumbbl.ffb.server.step.StepAction;
import com.fumbbl.ffb.server.step.StepId;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

/**
 * Mirror of ffb-rust crates/ffb-engine/src/step/bb2025/start/step_master_chef.rs (guard subset).
 * start() always → NEXT_STEP; the master-chef reroll-steal only runs on the first turn of a half in
 * halves 1-2, so half ≥ 3 short-circuits it (rerolls unchanged). The chef roll / event / report /
 * reroll-floor / first-turn-of-half detail tests are dice-driven and deferred.
 */
public class StepMasterChefFixtureTest {

	private GameState gameState;
	private Game game;

	@BeforeEach
	public void setUp() {
		gameState = GameFixture.createGameState(3, RulesCollection.Rules.BB2025);
		game = gameState.getGame();
	}

	private IStep newStep() {
		return GameFixture.createStep(gameState, StepId.MASTER_CHEF);
	}

	// rust: start_returns_next_step
	@Test
	public void startReturnsNextStep() {
		assertEquals(StepAction.NEXT_STEP, GameFixture.startStep(newStep()));
	}

	// rust: guard_skips_when_half_ge_3
	@Test
	public void guardSkipsWhenHalfGe3() {
		game.setHalf(3);
		int homeBefore = game.getTurnDataHome().getReRolls();
		int awayBefore = game.getTurnDataAway().getReRolls();
		assertEquals(StepAction.NEXT_STEP, GameFixture.startStep(newStep()));
		assertEquals(homeBefore, game.getTurnDataHome().getReRolls());
		assertEquals(awayBefore, game.getTurnDataAway().getReRolls());
	}
}
