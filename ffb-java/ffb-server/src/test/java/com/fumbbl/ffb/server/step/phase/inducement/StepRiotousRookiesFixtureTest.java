package com.fumbbl.ffb.server.step.phase.inducement;

import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.server.GameState;
import com.fumbbl.ffb.server.fixture.GameFixture;
import com.fumbbl.ffb.server.step.IStep;
import com.fumbbl.ffb.server.step.StepAction;
import com.fumbbl.ffb.server.step.StepId;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

/**
 * Mirror of ffb-rust crates/ffb-engine/src/step/phase/inducement/step_riotous_rookies.rs (guard).
 * start() hires the riotous rookies for both teams and returns NEXT_STEP. The created-journeyman
 * inspection tests (count 3-7 roll, journeyman status / RIOTOUS_ROOKIE type / Loner skill / id-format /
 * nr / reserves-box placement / position stat copy) are dice-driven + player-construction and deferred.
 */
public class StepRiotousRookiesFixtureTest {

	private GameState gameState;

	@BeforeEach
	public void setUp() {
		gameState = GameFixture.createGameState(3, RulesCollection.Rules.BB2016);
	}

	// rust: start_returns_next_step
	@Test
	public void startReturnsNextStep() {
		IStep step = GameFixture.createStep(gameState, StepId.RIOTOUS_ROOKIES);
		assertEquals(StepAction.NEXT_STEP, GameFixture.startStep(step));
	}
}
