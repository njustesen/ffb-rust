package com.fumbbl.ffb.server.step;

import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.server.GameState;
import com.fumbbl.ffb.server.fixture.GameFixture;
import com.fumbbl.ffb.server.skillbehaviour.StepHook.HookPoint;
import org.junit.jupiter.api.Test;

import java.util.List;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirror of ffb-rust crates/ffb-engine/src/skill_behaviour/step_hook.rs hooked_steps tests. The Rust
 * hooked_steps(rules, HookPoint) static table is the reflection-free analogue of Java's
 * StepFactory.getSteps(HookPoint) — which is populated by reflection-scanning every @StepHook IStep
 * class, edition-filtered by @RulesCollection. This asserts the Rust table matches what the real Java
 * StepFactory produces: exactly StepSafeThrow for BB2016 (PASS_INTERCEPT), StepCloudBurster for
 * BB2020, and nothing for BB2025 (neither annotated class matches that edition). (The Rust
 * hooked_steps_common_returns_none case uses Rules::Common, which has no Java game-options equivalent
 * — exempt; the HookPoint enum derive/StepHookHandler-trait tests are Rust-structural.)
 */
public class StepHookTest {

	private List<StepId> passInterceptSteps(RulesCollection.Rules rules) {
		GameState gameState = GameFixture.createGameState(3, rules);
		return gameState.getStepFactory().getSteps(HookPoint.PASS_INTERCEPT);
	}

	// rust: hooked_steps_bb2016_returns_safe_throw
	@Test
	public void hookedStepsBb2016ReturnsSafeThrow() {
		assertEquals(List.of(StepId.SAFE_THROW), passInterceptSteps(RulesCollection.Rules.BB2016));
	}

	// rust: hooked_steps_bb2020_returns_cloud_burster
	@Test
	public void hookedStepsBb2020ReturnsCloudBurster() {
		assertEquals(List.of(StepId.CLOUD_BURSTER), passInterceptSteps(RulesCollection.Rules.BB2020));
	}

	// rust: hooked_steps_bb2025_returns_none
	@Test
	public void hookedStepsBb2025ReturnsNone() {
		assertTrue(passInterceptSteps(RulesCollection.Rules.BB2025).isEmpty());
	}
}
