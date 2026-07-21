package com.fumbbl.ffb.server.step.generator.bb2025;

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
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirrors the Rust push-order tests in
 * {@code ffb-rust/crates/ffb-engine/src/step/generator/bb2025/select_blitz_target.rs}.
 */
public class SelectBlitzTargetFixtureTest {

	private GameState gameState;

	@BeforeEach
	public void setUp() {
		gameState = GameFixture.createGameState(3);
	}

	private IStep[] build() {
		new SelectBlitzTarget().pushSequence(new SequenceGenerator.SequenceParams(gameState));
		return GeneratorTestSupport.sequence(gameState);
	}

	// Rust: select_blitz_target_has_17_steps_with_activation
	@Test
	public void selectBlitzTargetHas17StepsWithActivation() {
		IStep[] steps = build();
		assertEquals(17, steps.length);
		assertEquals(StepId.INIT_ACTIVATION, steps[1].getId());
	}

	// Rust: select_blitz_target_is_labelled_select
	@Test
	public void selectBlitzTargetIsLabelledSelect() {
		IStep s = GeneratorTestSupport.find(build(), StepId.SELECT_BLITZ_TARGET);
		assertEquals(IStepLabel.SELECT, s.getLabel());
	}

	// Rust: select_blitz_target_end_is_labelled_end_blitzing
	@Test
	public void selectBlitzTargetEndIsLabelledEndBlitzing() {
		IStep s = GeneratorTestSupport.find(build(), StepId.SELECT_BLITZ_TARGET_END);
		assertEquals(IStepLabel.END_BLITZING, s.getLabel());
	}

	// Rust: select_blitz_target_has_jump_up_and_stand_up
	@Test
	public void selectBlitzTargetHasJumpUpAndStandUp() {
		IStep[] steps = build();
		assertTrue(GeneratorTestSupport.contains(steps, StepId.JUMP_UP));
		assertTrue(GeneratorTestSupport.contains(steps, StepId.STAND_UP));
	}

	// Rust: build_sequence_returns_vec
	@Test
	public void buildSequenceReturnsVec() {
		assertTrue(build().length > 0);
	}
}
