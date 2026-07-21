package com.fumbbl.ffb.server.step.generator.bb2025;

import com.fumbbl.ffb.server.GameState;
import com.fumbbl.ffb.server.fixture.GameFixture;
import com.fumbbl.ffb.server.fixture.GeneratorTestSupport;
import com.fumbbl.ffb.server.step.IStep;
import com.fumbbl.ffb.server.step.IStepLabel;
import com.fumbbl.ffb.server.step.StepId;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

/**
 * Mirrors the Rust push-order tests in
 * {@code ffb-rust/crates/ffb-engine/src/step/generator/bb2025/baleful_hex.rs}.
 */
public class BalefulHexFixtureTest {

	private GameState gameState;

	@BeforeEach
	public void setUp() {
		gameState = GameFixture.createGameState(3);
	}

	private IStep[] build(String failureLabel) {
		new BalefulHex().pushSequence(new BalefulHex.SequenceParams(gameState, failureLabel));
		return GeneratorTestSupport.sequence(gameState);
	}

	// Rust: baleful_hex_last_step_labelled_end
	@Test
	public void balefulHexLastStepLabelledEnd() {
		IStep[] steps = build("X");
		IStep last = steps[steps.length - 1];
		assertEquals(StepId.BALEFUL_HEX, last.getId());
		assertEquals(IStepLabel.END, last.getLabel());
	}

	// Rust: failure_label_in_params
	@Test
	public void failureLabelInParams() {
		IStep[] steps = build("theLabel");
		assertEquals("theLabel", GeneratorTestSupport.readField(steps[steps.length - 1], "goToLabelOnFailure"));
	}

	// Rust: baleful_hex_step_count_is_fourteen_with_activation
	@Test
	public void balefulHexStepCountIsFourteenWithActivation() {
		IStep[] steps = build("X");
		assertEquals(14, steps.length);
		assertEquals(StepId.INIT_ACTIVATION, steps[0].getId());
	}
}
