package com.fumbbl.ffb.server.step.generator.bb2025;

import com.fumbbl.ffb.PlayerState;
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

/**
 * Mirrors the Rust push-order tests in
 * {@code ffb-rust/crates/ffb-engine/src/step/generator/bb2025/black_ink.rs}.
 */
public class BlackInkFixtureTest {

	private GameState gameState;

	@BeforeEach
	public void setUp() {
		gameState = GameFixture.createGameState(3);
	}

	private IStep[] build(String failureLabel, PlayerState oldPlayerState) {
		new BlackInk().pushSequence(new BlackInk.SequenceParams(gameState, failureLabel, oldPlayerState));
		return GeneratorTestSupport.sequence(gameState);
	}

	// Rust: black_ink_last_step_labelled_end
	@Test
	public void blackInkLastStepLabelledEnd() {
		IStep[] steps = build("X", null);
		IStep last = steps[steps.length - 1];
		assertEquals(StepId.BLACK_INK, last.getId());
		assertEquals(IStepLabel.END, last.getLabel());
	}

	// Rust: activation_sub_sequence_precedes_black_ink
	@Test
	public void activationSubSequencePrecedesBlackInk() {
		IStep[] steps = build("X", null);
		assertEquals(14, steps.length);
		assertEquals(StepId.INIT_ACTIVATION, steps[0].getId());
	}

	// Rust: failure_label_in_params
	@Test
	public void failureLabelInParams() {
		IStep[] steps = build("theLabel", null);
		assertEquals("theLabel", GeneratorTestSupport.readField(steps[steps.length - 1], "goToLabelOnFailure"));
	}

	// Rust: old_player_state_added_when_some
	@Test
	public void oldPlayerStateAddedWhenSome() {
		IStep[] steps = build("X", new PlayerState(PlayerState.STANDING));
		assertNotNull(GeneratorTestSupport.readField(steps[steps.length - 1], "oldPlayerState"));
	}
}
