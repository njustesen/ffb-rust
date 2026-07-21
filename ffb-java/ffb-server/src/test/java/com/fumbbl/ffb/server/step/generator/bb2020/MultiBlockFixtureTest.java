package com.fumbbl.ffb.server.step.generator.bb2020;

import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.model.BlockTarget;
import com.fumbbl.ffb.server.GameState;
import com.fumbbl.ffb.server.fixture.GameFixture;
import com.fumbbl.ffb.server.fixture.GeneratorTestSupport;
import com.fumbbl.ffb.server.step.IStep;
import com.fumbbl.ffb.server.step.IStepLabel;
import com.fumbbl.ffb.server.step.StepId;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import java.util.ArrayList;
import java.util.List;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNotNull;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirrors the Rust push-order tests in
 * {@code ffb-rust/crates/ffb-engine/src/step/generator/bb2020/multi_block.rs}.
 */
public class MultiBlockFixtureTest {

	private GameState gameState;

	@BeforeEach
	public void setUp() {
		gameState = GameFixture.createGameState(3, RulesCollection.Rules.BB2020);
	}

	private IStep[] build() {
		List<BlockTarget> targets = new ArrayList<>();
		new MultiBlock().pushSequence(new MultiBlock.SequenceParams(gameState, targets));
		return GeneratorTestSupport.sequence(gameState);
	}

	// Rust: multi_block_has_activation_block
	@Test
	public void multiBlockHasActivationBlock() {
		IStep[] steps = build();
		assertTrue(GeneratorTestSupport.contains(steps, StepId.INIT_ACTIVATION));
		assertTrue(GeneratorTestSupport.contains(steps, StepId.BONE_HEAD));
	}

	// Rust: multi_block_ends_with_end_blocking_labelled
	@Test
	public void multiBlockEndsWithEndBlockingLabelled() {
		IStep[] steps = build();
		IStep last = steps[steps.length - 1];
		assertEquals(StepId.END_BLOCKING, last.getId());
		assertEquals(IStepLabel.END_BLOCKING, last.getLabel());
	}

	// Rust: multi_block_catch_scatter_labelled_scatter_ball
	@Test
	public void multiBlockCatchScatterLabelledScatterBall() {
		assertNotNull(GeneratorTestSupport.findLabelled(build(),
			StepId.CATCH_SCATTER_THROW_IN, IStepLabel.SCATTER_BALL));
	}

	// Rust: multi_block_apothecary_multiple_both
	@Test
	public void multiBlockApothecaryMultipleBoth() {
		assertEquals(2, GeneratorTestSupport.count(build(), StepId.APOTHECARY_MULTIPLE));
	}

	// Rust: multi_block_blood_lust_has_failure_label_end_blocking
	@Test
	public void multiBlockBloodLustHasFailureLabelEndBlocking() {
		IStep bloodLust = GeneratorTestSupport.find(build(), StepId.BLOOD_LUST);
		Object state = GeneratorTestSupport.readField(bloodLust, "state");
		assertNotNull(GeneratorTestSupport.readField(state, "goToLabelOnFailure"));
	}
}
