package com.fumbbl.ffb.server.step.generator.bb2025;

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
 * {@code ffb-rust/crates/ffb-engine/src/step/generator/bb2025/multi_block.rs}.
 * The step count is target-independent (22), so an empty target list is used.
 */
public class MultiBlockFixtureTest {

	private GameState gameState;

	@BeforeEach
	public void setUp() {
		gameState = GameFixture.createGameState(3);
	}

	private IStep[] build() {
		List<BlockTarget> targets = new ArrayList<>();
		new MultiBlock().pushSequence(new MultiBlock.SequenceParams(gameState, targets));
		return GeneratorTestSupport.sequence(gameState);
	}

	// Rust: multi_block_has_22_steps / multi_block_empty_targets_has_22_steps
	@Test
	public void multiBlockHas22Steps() {
		assertEquals(22, build().length);
	}

	// Rust: multi_block_starts_with_activation_sequence
	@Test
	public void multiBlockStartsWithActivationSequence() {
		IStep[] steps = build();
		assertEquals(StepId.INIT_ACTIVATION, steps[0].getId());
		assertTrue(GeneratorTestSupport.contains(steps, StepId.BONE_HEAD));
		assertTrue(GeneratorTestSupport.contains(steps, StepId.REALLY_STUPID));
		assertTrue(GeneratorTestSupport.contains(steps, StepId.TAKE_ROOT));
		assertTrue(GeneratorTestSupport.contains(steps, StepId.UNCHANNELLED_FURY));
		assertTrue(GeneratorTestSupport.contains(steps, StepId.BLOOD_LUST));
		assertTrue(GeneratorTestSupport.contains(steps, StepId.ANIMAL_SAVAGERY));
		assertTrue(GeneratorTestSupport.indexOf(steps, StepId.INIT_ACTIVATION)
			< GeneratorTestSupport.indexOf(steps, StepId.FOUL_APPEARANCE_MULTIPLE));
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

	// Rust: multi_block_apothecary_multiple_acting_team_both
	@Test
	public void multiBlockApothecaryMultipleActingTeamBoth() {
		assertEquals(2, GeneratorTestSupport.count(build(), StepId.APOTHECARY_MULTIPLE));
	}
}
