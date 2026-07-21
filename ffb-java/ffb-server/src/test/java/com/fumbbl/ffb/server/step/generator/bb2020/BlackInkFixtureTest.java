package com.fumbbl.ffb.server.step.generator.bb2020;

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
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirrors the Rust push-order tests in
 * {@code ffb-rust/crates/ffb-engine/src/step/generator/bb2020/black_ink.rs}.
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

	// Rust: black_ink_has_activation_block
	@Test
	public void blackInkHasActivationBlock() {
		assertTrue(GeneratorTestSupport.contains(build("X", null), StepId.INIT_ACTIVATION));
	}

	// Rust: black_ink_ends_with_black_ink_labelled_end
	@Test
	public void blackInkEndsWithBlackInkLabelledEnd() {
		IStep[] steps = build("X", null);
		IStep last = steps[steps.length - 1];
		assertEquals(StepId.BLACK_INK, last.getId());
		assertEquals(IStepLabel.END, last.getLabel());
	}

	// Rust: black_ink_has_foul_appearance
	@Test
	public void blackInkHasFoulAppearance() {
		assertTrue(GeneratorTestSupport.contains(build("X", null), StepId.FOUL_APPEARANCE));
	}

	// Rust: black_ink_blood_lust_has_no_failure_label — NOT mirrored (nested state field).

	// Rust: failure_label_passed_to_black_ink_step
	@Test
	public void failureLabelPassedToBlackInkStep() {
		IStep ink = GeneratorTestSupport.find(build("theLabel", null), StepId.BLACK_INK);
		assertEquals("theLabel", GeneratorTestSupport.readField(ink, "goToLabelOnFailure"));
	}

	// Rust: old_player_state_added_when_some
	@Test
	public void oldPlayerStateAddedWhenSome() {
		IStep ink = GeneratorTestSupport.find(build("X", new PlayerState(PlayerState.STANDING)), StepId.BLACK_INK);
		assertNotNull(GeneratorTestSupport.readField(ink, "oldPlayerState"));
	}
}
