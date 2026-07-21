package com.fumbbl.ffb.server.step.generator.bb2020;

import com.fumbbl.ffb.RulesCollection;
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
import static org.junit.jupiter.api.Assertions.assertNotNull;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirrors the Rust push-order tests in
 * {@code ffb-rust/crates/ffb-engine/src/step/generator/bb2020/select_blitz_target.rs}.
 */
public class SelectBlitzTargetFixtureTest {

	private GameState gameState;

	@BeforeEach
	public void setUp() {
		gameState = GameFixture.createGameState(3, RulesCollection.Rules.BB2020);
	}

	private IStep[] build() {
		new SelectBlitzTarget().pushSequence(new SequenceGenerator.SequenceParams(gameState));
		return GeneratorTestSupport.sequence(gameState);
	}

	// Rust: select_blitz_target_is_labelled_select
	@Test
	public void selectBlitzTargetIsLabelledSelect() {
		assertEquals(IStepLabel.SELECT,
			GeneratorTestSupport.find(build(), StepId.SELECT_BLITZ_TARGET).getLabel());
	}

	// Rust: select_blitz_target_end_is_labelled_end_blitzing
	@Test
	public void selectBlitzTargetEndIsLabelledEndBlitzing() {
		assertEquals(IStepLabel.END_BLITZING,
			GeneratorTestSupport.find(build(), StepId.SELECT_BLITZ_TARGET_END).getLabel());
	}

	// Rust: select_blitz_target_has_activation_block
	@Test
	public void selectBlitzTargetHasActivationBlock() {
		assertTrue(GeneratorTestSupport.contains(build(), StepId.INIT_ACTIVATION));
	}

	// Rust: select_blitz_target_has_dump_off
	@Test
	public void selectBlitzTargetHasDumpOff() {
		assertTrue(GeneratorTestSupport.contains(build(), StepId.DUMP_OFF));
	}

	// Rust: select_blitz_target_blood_lust_has_failure_label
	@Test
	public void selectBlitzTargetBloodLustHasFailureLabel() {
		IStep bloodLust = GeneratorTestSupport.find(build(), StepId.BLOOD_LUST);
		Object state = GeneratorTestSupport.readField(bloodLust, "state");
		assertNotNull(GeneratorTestSupport.readField(state, "goToLabelOnFailure"));
	}

	// Rust: select_blitz_target_step_count
	@Test
	public void selectBlitzTargetStepCount() {
		assertEquals(18, build().length);
	}
}
