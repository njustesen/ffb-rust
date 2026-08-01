package com.fumbbl.ffb.server.step.mixed.shared;

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
 * Mirror of ffb-rust crates/ffb-engine/src/step/mixed/shared/step_pick_me_up.rs (last-turn guard).
 * On the first run, a touchdown or the final turn (both teams at turn 8) short-circuits Pick-Me-Up to
 * NEXT_STEP. The touchdown-skip / eligible-player collection / player-choice / roll / report tests
 * need touchdown state or dialog commands and are deferred.
 */
public class StepPickMeUpFixtureTest {

	private GameState gameState;
	private Game game;

	@BeforeEach
	public void setUp() {
		gameState = GameFixture.createGameState(3, RulesCollection.Rules.BB2020);
		game = gameState.getGame();
	}

	// rust: last_turn_skips_pick_me_up
	@Test
	public void lastTurnSkipsPickMeUp() {
		game.getTurnDataHome().setTurnNr(8);
		game.getTurnDataAway().setTurnNr(8);
		IStep step = GameFixture.createStep(gameState, StepId.PICK_ME_UP);
		assertEquals(StepAction.NEXT_STEP, GameFixture.startStep(step));
	}
}
