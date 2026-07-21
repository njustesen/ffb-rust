package com.fumbbl.ffb.server.step.generator.bb2025;

import com.fumbbl.ffb.server.GameState;
import com.fumbbl.ffb.server.fixture.GameFixture;
import com.fumbbl.ffb.server.step.IStep;
import com.fumbbl.ffb.server.step.IStepLabel;
import com.fumbbl.ffb.server.step.StepId;
import com.fumbbl.ffb.server.step.generator.GeneratorTestSupport;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertNotNull;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirrors the Rust push-order tests in
 * {@code ffb-rust/crates/ffb-engine/src/step/generator/bb2025/blitz_block.rs}.
 */
public class BlitzBlockFixtureTest {

	private GameState gameState;

	@BeforeEach
	public void setUp() {
		gameState = GameFixture.createGameState(3);
	}

	private IStep[] build() {
		new BlitzBlock().pushSequence(new BlitzBlock.SequenceParams(gameState));
		return GeneratorTestSupport.sequence(gameState);
	}

	// Rust: blitz_block_starts_with_init_blocking
	@Test
	public void blitzBlockStartsWithInitBlocking() {
		assertEquals(StepId.INIT_BLOCKING, build()[0].getId());
	}

	// Rust: blitz_block_ends_with_end_blocking
	@Test
	public void blitzBlockEndsWithEndBlocking() {
		IStep[] steps = build();
		assertEquals(StepId.END_BLOCKING, steps[steps.length - 1].getId());
	}

	// Rust: blitz_block_remove_target_selection_state_is_labelled_end_blocking
	@Test
	public void blitzBlockRemoveTargetSelectionStateIsLabelledEndBlocking() {
		assertNotNull(GeneratorTestSupport.findLabelled(build(),
			StepId.REMOVE_TARGET_SELECTION_STATE, IStepLabel.END_BLOCKING));
	}

	// Rust: blitz_block_has_no_activation_block
	@Test
	public void blitzBlockHasNoActivationBlock() {
		assertFalse(GeneratorTestSupport.contains(build(), StepId.INIT_ACTIVATION));
	}

	// Rust: blitz_block_has_gfi_before_foul_appearance
	@Test
	public void blitzBlockHasGfiBeforeFoulAppearance() {
		IStep[] steps = build();
		assertTrue(GeneratorTestSupport.indexOf(steps, StepId.GO_FOR_IT)
			< GeneratorTestSupport.indexOf(steps, StepId.FOUL_APPEARANCE));
	}

	// Rust: blitz_block_has_horns
	@Test
	public void blitzBlockHasHorns() {
		assertTrue(GeneratorTestSupport.contains(build(), StepId.HORNS));
	}

	// Rust: blitz_block_steady_footing_labelled_steady_footing_before_foul_appearance
	@Test
	public void blitzBlockSteadyFootingLabelledSteadyFootingBeforeFoulAppearance() {
		IStep[] steps = build();
		IStep sf = GeneratorTestSupport.findLabelled(steps, StepId.STEADY_FOOTING, IStepLabel.STEADY_FOOTING);
		assertNotNull(sf);
		int sfIdx = GeneratorTestSupport.indexOfInstance(steps, sf);
		int faIdx = GeneratorTestSupport.indexOf(steps, StepId.FOUL_APPEARANCE);
		assertTrue(sfIdx < faIdx);
	}

	// Rust: blitz_block_fall_down_is_labelled_fall_down
	@Test
	public void blitzBlockFallDownIsLabelledFallDown() {
		IStep fd = GeneratorTestSupport.find(build(), StepId.FALL_DOWN);
		assertNotNull(fd);
		assertEquals(IStepLabel.FALL_DOWN, fd.getLabel());
	}

	// Rust: blitz_block_drop_falling_players_is_labelled
	@Test
	public void blitzBlockDropFallingPlayersIsLabelled() {
		assertNotNull(GeneratorTestSupport.findLabelled(build(),
			StepId.DROP_FALLING_PLAYERS, IStepLabel.DROP_FALLING_PLAYERS));
	}

	// Rust: blitz_block_has_51_steps
	@Test
	public void blitzBlockHas51Steps() {
		assertEquals(51, build().length);
	}

	// Rust: blitz_block_reset_fumblerooskie_has_failed_block_param
	@Test
	public void blitzBlockResetFumblerooskieHasFailedBlockParam() {
		IStep rf = GeneratorTestSupport.find(build(), StepId.RESET_FUMBLEROOSKIE);
		assertNotNull(rf);
		assertTrue(GeneratorTestSupport.booleanField(rf, "resetForFailedBlock"));
	}
}
