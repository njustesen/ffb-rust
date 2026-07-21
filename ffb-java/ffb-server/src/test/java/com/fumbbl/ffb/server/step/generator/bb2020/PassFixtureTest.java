package com.fumbbl.ffb.server.step.generator.bb2020;

import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.server.GameState;
import com.fumbbl.ffb.server.fixture.GameFixture;
import com.fumbbl.ffb.server.fixture.GeneratorTestSupport;
import com.fumbbl.ffb.server.step.IStep;
import com.fumbbl.ffb.server.step.IStepLabel;
import com.fumbbl.ffb.server.step.StepId;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNotNull;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirrors the Rust push-order tests in
 * {@code ffb-rust/crates/ffb-engine/src/step/generator/bb2020/pass.rs}.
 */
public class PassFixtureTest {

	private GameState gameState;

	@BeforeEach
	public void setUp() {
		gameState = GameFixture.createGameState(3, RulesCollection.Rules.BB2020);
	}

	private IStep[] build() {
		new Pass().pushSequence(new Pass.SequenceParams(gameState));
		return GeneratorTestSupport.sequence(gameState);
	}

	// Rust: pass_sequence_starts_with_init_passing
	@Test
	public void passSequenceStartsWithInitPassing() {
		assertEquals(StepId.INIT_PASSING, build()[0].getId());
	}

	// Rust: pass_sequence_ends_with_end_passing
	@Test
	public void passSequenceEndsWithEndPassing() {
		IStep[] steps = build();
		assertEquals(StepId.END_PASSING, steps[steps.length - 1].getId());
	}

	// Rust: pass_has_activation_block
	@Test
	public void passHasActivationBlock() {
		assertTrue(GeneratorTestSupport.contains(build(), StepId.INIT_ACTIVATION));
	}

	// Rust: pass_intercept_is_labelled
	@Test
	public void passInterceptIsLabelled() {
		assertEquals(IStepLabel.INTERCEPT, GeneratorTestSupport.find(build(), StepId.INTERCEPT).getLabel());
	}

	// Rust: pass_catch_scatter_labelled_scatter_ball
	@Test
	public void passCatchScatterLabelledScatterBall() {
		assertNotNull(GeneratorTestSupport.findLabelled(build(),
			StepId.CATCH_SCATTER_THROW_IN, IStepLabel.SCATTER_BALL));
	}

	// Rust: pass_inserts_cloud_burster_hook_after_intercept
	@Test
	public void passInsertsCloudBursterHookAfterIntercept() {
		IStep[] steps = build();
		int interceptIdx = GeneratorTestSupport.indexOf(steps, StepId.INTERCEPT);
		IStep cloudBurster = steps[interceptIdx + 1];
		assertEquals(StepId.CLOUD_BURSTER, cloudBurster.getId());
		assertEquals(IStepLabel.RESOLVE_PASS, GeneratorTestSupport.readField(cloudBurster, "fGotoLabelOnFailure"));
	}
}
