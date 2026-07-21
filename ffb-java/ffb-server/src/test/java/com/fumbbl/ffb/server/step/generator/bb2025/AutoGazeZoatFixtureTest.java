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
import static org.junit.jupiter.api.Assertions.assertNull;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirrors the Rust push-order tests in
 * {@code ffb-rust/crates/ffb-engine/src/step/generator/bb2025/auto_gaze_zoat.rs}.
 */
public class AutoGazeZoatFixtureTest {

	private GameState gameState;

	@BeforeEach
	public void setUp() {
		gameState = GameFixture.createGameState(3);
	}

	private IStep[] build(String failureLabel, PlayerState oldPlayerState) {
		new AutoGazeZoat().pushSequence(new AutoGazeZoat.SequenceParams(gameState, failureLabel, oldPlayerState));
		return GeneratorTestSupport.sequence(gameState);
	}

	// Rust: auto_gaze_zoat_last_step_labelled_end
	@Test
	public void autoGazeZoatLastStepLabelledEnd() {
		IStep[] steps = build("someLabel", null);
		IStep last = steps[steps.length - 1];
		assertEquals(StepId.AUTO_GAZE_ZOAT, last.getId());
		assertEquals(IStepLabel.END, last.getLabel());
	}

	// Rust: activation_sub_sequence_precedes_auto_gaze_zoat
	@Test
	public void activationSubSequencePrecedesAutoGazeZoat() {
		IStep[] steps = build("X", null);
		assertEquals(14, steps.length);
		assertEquals(StepId.INIT_ACTIVATION, steps[0].getId());
		assertEquals(StepId.AUTO_GAZE_ZOAT, steps[13].getId());
	}

	// Rust: failure_label_passed_as_goto_label_on_failure
	@Test
	public void failureLabelPassedAsGotoLabelOnFailure() {
		IStep[] steps = build("myLabel", null);
		assertEquals("myLabel", GeneratorTestSupport.readField(steps[steps.length - 1], "goToLabelOnFailure"));
	}

	// Rust: old_player_state_added_when_some
	@Test
	public void oldPlayerStateAddedWhenSome() {
		IStep[] steps = build("X", new PlayerState(PlayerState.STANDING));
		assertNotNull(GeneratorTestSupport.readField(steps[steps.length - 1], "oldPlayerState"));
	}

	// Rust: old_player_state_absent_when_none
	@Test
	public void oldPlayerStateAbsentWhenNone() {
		IStep[] steps = build("X", null);
		assertNull(GeneratorTestSupport.readField(steps[steps.length - 1], "oldPlayerState"));
	}

	// Rust: build_sequence_is_nonempty
	@Test
	public void buildSequenceIsNonempty() {
		assertTrue(build("X", null).length > 0);
	}
}
