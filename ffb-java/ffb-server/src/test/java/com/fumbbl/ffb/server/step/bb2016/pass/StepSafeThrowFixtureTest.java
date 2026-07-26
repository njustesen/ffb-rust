package com.fumbbl.ffb.server.step.bb2016.pass;

import com.fumbbl.ffb.PlayerAction;
import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.server.GameState;
import com.fumbbl.ffb.server.fixture.GameFixture;
import com.fumbbl.ffb.server.step.IStep;
import com.fumbbl.ffb.server.step.StepAction;
import com.fumbbl.ffb.server.step.StepId;
import com.fumbbl.ffb.server.step.StepParameter;
import com.fumbbl.ffb.server.step.StepParameterKey;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirrors the Rust tests in
 * {@code ffb-rust/crates/ffb-engine/src/step/bb2016/pass/step_safe_throw.rs} (param + next-step
 * subset). INTERCEPTOR_ID (including null) is stored via setParameter. With an interceptor but a
 * thrower lacking the Safe Throw skill, doSafeThrow is false and the step falls through to the failure
 * branch (interception stands, ball goes to the interceptor) -> GOTO_LABEL. This exposed a Rust
 * translation bug: Rust's step_safe_throw early-returned NEXT_STEP for the no-safe-throw and
 * VeryLongLegs-cancel cases; fixed in Rust to call fail_safe_throw (GOTO_LABEL) matching this ground
 * truth. no_interceptor_returns_next is exempt (Java StepSafeThrow dequeues a command first, so with
 * no pending command it returns CONTINUE, not the Rust NEXT_STEP — command-loop divergence).
 * GOTO_LABEL_ON_FAILURE is init-consumed (setParameter false, exempt); the safe-throw-failure
 * ball/bomb coordinate, roll-report, and decline-reroll tests need dice outcomes / published-param or
 * report inspection and are deferred.
 */
public class StepSafeThrowFixtureTest {

	private GameState gameState;

	@BeforeEach
	public void setUp() {
		gameState = GameFixture.createGameState(3, RulesCollection.Rules.BB2016);
		gameState.getGame().setHomePlaying(true);
	}

	private IStep newStep() {
		return GameFixture.createStep(gameState, StepId.SAFE_THROW);
	}

	// rust: set_parameter_interceptor_id
	@Test
	public void setParameterInterceptorId() {
		assertTrue(newStep().setParameter(StepParameter.from(StepParameterKey.INTERCEPTOR_ID, "away1")));
	}

	// rust: set_parameter_interceptor_id_none
	@Test
	public void setParameterInterceptorIdNone() {
		assertTrue(newStep().setParameter(StepParameter.from(StepParameterKey.INTERCEPTOR_ID, null)));
	}

	// rust: interceptor_but_thrower_lacks_safe_throw_skill_goes_to_failure
	@Test
	public void interceptorButThrowerLacksSafeThrowSkillGoesToFailure() {
		GameFixture.placePlayer(gameState, "home1", 5, 5);
		GameFixture.setActingPlayer(gameState, "home1", PlayerAction.PASS);
		gameState.getGame().setThrowerId("home1");
		GameFixture.placePlayer(gameState, "away1", 8, 5);
		IStep step = newStep();
		step.setParameter(StepParameter.from(StepParameterKey.INTERCEPTOR_ID, "away1"));
		assertEquals(StepAction.GOTO_LABEL, GameFixture.startStep(step));
	}
}
