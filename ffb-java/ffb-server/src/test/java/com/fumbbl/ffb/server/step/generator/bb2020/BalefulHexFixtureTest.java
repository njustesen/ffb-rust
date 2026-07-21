package com.fumbbl.ffb.server.step.generator.bb2020;

import com.fumbbl.ffb.server.GameState;
import com.fumbbl.ffb.server.fixture.GameFixture;
import com.fumbbl.ffb.server.fixture.GeneratorTestSupport;
import com.fumbbl.ffb.server.step.IStep;
import com.fumbbl.ffb.server.step.IStepLabel;
import com.fumbbl.ffb.server.step.StepId;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNull;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirrors the Rust push-order tests in
 * {@code ffb-rust/crates/ffb-engine/src/step/generator/bb2020/baleful_hex.rs}.
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

	// Rust: baleful_hex_has_activation_block
	@Test
	public void balefulHexHasActivationBlock() {
		assertTrue(GeneratorTestSupport.contains(build("X"), StepId.INIT_ACTIVATION));
	}

	// Rust: baleful_hex_ends_with_baleful_hex_labelled_end
	@Test
	public void balefulHexEndsWithBalefulHexLabelledEnd() {
		IStep[] steps = build("X");
		IStep last = steps[steps.length - 1];
		assertEquals(StepId.BALEFUL_HEX, last.getId());
		assertEquals(IStepLabel.END, last.getLabel());
	}

	// Rust: baleful_hex_bone_head_has_no_label
	@Test
	public void balefulHexBoneHeadHasNoLabel() {
		IStep boneHead = GeneratorTestSupport.find(build("X"), StepId.BONE_HEAD);
		assertNull(boneHead.getLabel());
	}

	// Rust: baleful_hex_blood_lust_has_no_failure_label — NOT mirrored.
	// The Rust test asserts the BloodLust SequenceStep has no GOTO_LABEL_ON_FAILURE param.
	// Java's StepBloodLust consumes that param into a nested `state.goToLabelOnFailure`
	// field, so param-absence is not cleanly observable via the built step. Structural
	// presence of the (unlabelled) activation sub-sequence is covered by the other tests.

	// Rust: build_sequence_is_nonempty
	@Test
	public void buildSequenceIsNonempty() {
		assertTrue(build("X").length > 0);
	}
}
