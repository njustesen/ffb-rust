package com.fumbbl.ffb.server.step.generator.bb2020;

import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.server.GameState;
import com.fumbbl.ffb.server.fixture.GameFixture;
import com.fumbbl.ffb.server.fixture.GeneratorTestSupport;
import com.fumbbl.ffb.server.step.IStep;
import com.fumbbl.ffb.server.step.IStepLabel;
import com.fumbbl.ffb.server.step.StepId;
import com.fumbbl.ffb.server.step.generator.SequenceGenerator;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertNull;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirrors the Rust push-order tests in
 * {@code ffb-rust/crates/ffb-engine/src/step/generator/bb2020/then_i_started_blastin.rs}.
 */
public class ThenIStartedBlastinFixtureTest {

	private GameState gameState;

	@BeforeEach
	public void setUp() {
		gameState = GameFixture.createGameState(3, RulesCollection.Rules.BB2020);
	}

	private IStep[] build() {
		new ThenIStartedBlastin().pushSequence(new SequenceGenerator.SequenceParams(gameState));
		return GeneratorTestSupport.sequence(gameState);
	}

	// Rust: then_i_started_blastin_has_activation_block
	@Test
	public void thenIStartedBlastinHasActivationBlock() {
		assertTrue(GeneratorTestSupport.contains(build(), StepId.INIT_ACTIVATION));
	}

	// Rust: then_i_started_blastin_ends_labelled_end
	@Test
	public void thenIStartedBlastinEndsLabelledEnd() {
		IStep[] steps = build();
		IStep last = steps[steps.length - 1];
		assertEquals(StepId.END_THEN_I_STARTED_BLASTIN, last.getId());
		assertEquals(IStepLabel.END, last.getLabel());
	}

	// Rust: then_i_started_blastin_bone_head_has_no_label
	@Test
	public void thenIStartedBlastinBoneHeadHasNoLabel() {
		assertNull(GeneratorTestSupport.find(build(), StepId.BONE_HEAD).getLabel());
	}

	// Rust: then_i_started_blastin_has_no_steady_footing
	@Test
	public void thenIStartedBlastinHasNoSteadyFooting() {
		assertFalse(GeneratorTestSupport.contains(build(), StepId.STEADY_FOOTING));
	}

	// Rust: then_i_started_blastin_step_count
	@Test
	public void thenIStartedBlastinStepCount() {
		assertEquals(16, build().length);
	}

	// Rust: then_i_started_blastin_has_then_i_started_blastin_step
	@Test
	public void thenIStartedBlastinHasThenIStartedBlastinStep() {
		assertTrue(GeneratorTestSupport.contains(build(), StepId.THEN_I_STARTED_BLASTIN));
	}
}
