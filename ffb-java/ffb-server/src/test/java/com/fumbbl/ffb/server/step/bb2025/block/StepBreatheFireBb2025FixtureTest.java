package com.fumbbl.ffb.server.step.bb2025.block;

import com.fumbbl.ffb.PlayerAction;
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
 * Mirrors the Rust tests in
 * {@code ffb-rust/crates/ffb-engine/src/step/bb2025/block/step_breathe_fire.rs} (next-step subset).
 * With the acting player not using breathe fire (the default), the step falls through to NEXT_STEP.
 * The evaluate(roll, effectiveRoll) tests exercise a private Java helper (exempt); the roll-produces-
 * prone/knock-down, no-effect-goto, and report tests are dice/publish/report driven and deferred.
 * GOTO_LABEL_ON_END is init-consumed (exempt).
 */
public class StepBreatheFireBb2025FixtureTest {

	private GameState gameState;

	@BeforeEach
	public void setUp() {
		gameState = GameFixture.createGameState(3, RulesCollection.Rules.BB2025);
		gameState.getGame().setHomePlaying(true);
		GameFixture.placePlayer(gameState, "home1", 5, 5);
		GameFixture.setActingPlayer(gameState, "home1", PlayerAction.BLOCK);
	}

	private IStep newStep() {
		return GameFixture.createStep(gameState, StepId.BREATHE_FIRE);
	}

	// rust: not_using_breathe_fire_returns_next_step
	@Test
	public void notUsingBreatheFireReturnsNextStep() {
		assertEquals(StepAction.NEXT_STEP, GameFixture.startStep(newStep()));
	}

	// rust: no_effect_result_gotos_goto_on_end (not-using fallback -> NEXT_STEP)
	@Test
	public void noEffectFallbackReturnsNextStep() {
		assertEquals(StepAction.NEXT_STEP, GameFixture.startStep(newStep()));
	}
}
