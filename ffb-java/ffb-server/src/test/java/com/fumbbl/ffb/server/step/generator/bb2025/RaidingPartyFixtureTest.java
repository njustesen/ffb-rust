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
 * {@code ffb-rust/crates/ffb-engine/src/step/generator/bb2025/raiding_party.rs}.
 */
public class RaidingPartyFixtureTest {

	private GameState gameState;

	@BeforeEach
	public void setUp() {
		gameState = GameFixture.createGameState(3);
	}

	private IStep[] build(String failureLabel, String successLabel) {
		new RaidingParty().pushSequence(new RaidingParty.SequenceParams(gameState, failureLabel, successLabel));
		return GeneratorTestSupport.sequence(gameState);
	}

	// Rust: raiding_party_last_step_labelled_end
	@Test
	public void raidingPartyLastStepLabelledEnd() {
		IStep[] steps = build("fail", "ok");
		IStep last = steps[steps.length - 1];
		assertEquals(StepId.RAIDING_PARTY, last.getId());
		assertEquals(IStepLabel.END, last.getLabel());
	}

	// Rust: activation_sub_sequence_precedes_raiding_party
	@Test
	public void activationSubSequencePrecedesRaidingParty() {
		IStep[] steps = build("X", "Y");
		assertEquals(14, steps.length);
		assertEquals(StepId.INIT_ACTIVATION, steps[0].getId());
	}

	// Rust: failure_label_wired
	@Test
	public void failureLabelWired() {
		IStep[] steps = build("FAIL_LABEL", "OK");
		assertEquals("FAIL_LABEL", GeneratorTestSupport.readField(steps[steps.length - 1], "goToLabelOnFailure"));
	}

	// Rust: success_label_wired
	@Test
	public void successLabelWired() {
		IStep[] steps = build("F", "SUCCESS_LABEL");
		assertEquals("SUCCESS_LABEL", GeneratorTestSupport.readField(steps[steps.length - 1], "gotoLabelOnSuccess"));
	}
}
