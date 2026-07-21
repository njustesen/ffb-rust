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
 * {@code ffb-rust/crates/ffb-engine/src/step/generator/bb2020/raiding_party.rs}.
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

	// Rust: raiding_party_has_activation_block
	@Test
	public void raidingPartyHasActivationBlock() {
		assertTrue(GeneratorTestSupport.contains(build("f", "s"), StepId.INIT_ACTIVATION));
	}

	// Rust: raiding_party_ends_labelled_end
	@Test
	public void raidingPartyEndsLabelledEnd() {
		IStep[] steps = build("fail", "ok");
		IStep last = steps[steps.length - 1];
		assertEquals(StepId.RAIDING_PARTY, last.getId());
		assertEquals(IStepLabel.END, last.getLabel());
	}

	// Rust: raiding_party_bone_head_has_no_label
	@Test
	public void raidingPartyBoneHeadHasNoLabel() {
		assertNull(GeneratorTestSupport.find(build("f", "s"), StepId.BONE_HEAD).getLabel());
	}

	// Rust: raiding_party_blood_lust_has_no_failure_label — NOT mirrored
	// (StepBloodLust stores the label in a nested state field; param-absence not observable).

	// Rust: failure_label_passed_to_raiding_party_step
	@Test
	public void failureLabelPassedToRaidingPartyStep() {
		IStep rp = GeneratorTestSupport.find(build("fail_lbl", "ok"), StepId.RAIDING_PARTY);
		assertEquals("fail_lbl", GeneratorTestSupport.readField(rp, "goToLabelOnFailure"));
	}

	// Rust: success_label_passed_to_raiding_party_step
	@Test
	public void successLabelPassedToRaidingPartyStep() {
		IStep rp = GeneratorTestSupport.find(build("fail", "succ_lbl"), StepId.RAIDING_PARTY);
		assertEquals("succ_lbl", GeneratorTestSupport.readField(rp, "gotoLabelOnSuccess"));
	}
}
