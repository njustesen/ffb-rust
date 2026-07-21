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
 * {@code ffb-rust/crates/ffb-engine/src/step/generator/bb2020/treacherous.rs}.
 */
public class TreacherousFixtureTest {

	private GameState gameState;

	@BeforeEach
	public void setUp() {
		gameState = GameFixture.createGameState(3);
	}

	private IStep[] build(String failureLabel) {
		new Treacherous().pushSequence(new Treacherous.SequenceParams(gameState, failureLabel));
		return GeneratorTestSupport.sequence(gameState);
	}

	// Rust: treacherous_has_activation_block
	@Test
	public void treacherousHasActivationBlock() {
		assertTrue(GeneratorTestSupport.contains(build("X"), StepId.INIT_ACTIVATION));
	}

	// Rust: treacherous_is_labelled_end
	@Test
	public void treacherousIsLabelledEnd() {
		IStep t = GeneratorTestSupport.find(build("X"), StepId.TREACHEROUS);
		assertEquals(IStepLabel.END, t.getLabel());
	}

	// Rust: treacherous_bone_head_has_no_label
	@Test
	public void treacherousBoneHeadHasNoLabel() {
		assertNull(GeneratorTestSupport.find(build("X"), StepId.BONE_HEAD).getLabel());
	}

	// Rust: treacherous_ends_with_apothecary_defender
	@Test
	public void treacherousEndsWithApothecaryDefender() {
		IStep[] steps = build("X");
		assertEquals(StepId.APOTHECARY, steps[steps.length - 1].getId());
	}

	// Rust: failure_label_passed_to_treacherous_step
	@Test
	public void failureLabelPassedToTreacherousStep() {
		IStep t = GeneratorTestSupport.find(build("fail_here"), StepId.TREACHEROUS);
		assertEquals("fail_here", GeneratorTestSupport.readField(t, "goToLabelOnFailure"));
	}

	// Rust: treacherous_step_count
	@Test
	public void treacherousStepCount() {
		assertEquals(16, build("X").length);
	}
}
