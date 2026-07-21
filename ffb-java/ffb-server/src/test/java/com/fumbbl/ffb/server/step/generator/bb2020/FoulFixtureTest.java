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
import static org.junit.jupiter.api.Assertions.assertNotNull;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirrors the Rust push-order tests in
 * {@code ffb-rust/crates/ffb-engine/src/step/generator/bb2020/foul.rs}.
 */
public class FoulFixtureTest {

	private GameState gameState;

	@BeforeEach
	public void setUp() {
		gameState = GameFixture.createGameState(3);
	}

	private IStep[] build() {
		new Foul().pushSequence(new Foul.SequenceParams(gameState));
		return GeneratorTestSupport.sequence(gameState);
	}

	// Rust: foul_starts_with_init_fouling
	@Test
	public void foulStartsWithInitFouling() {
		assertEquals(StepId.INIT_FOULING, build()[0].getId());
	}

	// Rust: foul_ends_with_end_fouling
	@Test
	public void foulEndsWithEndFouling() {
		IStep[] steps = build();
		assertEquals(StepId.END_FOULING, steps[steps.length - 1].getId());
	}

	// Rust: foul_has_activation_block
	@Test
	public void foulHasActivationBlock() {
		assertTrue(GeneratorTestSupport.contains(build(), StepId.INIT_ACTIVATION));
	}

	// Rust: foul_apothecary_attacker_is_labelled
	@Test
	public void foulApothecaryAttackerIsLabelled() {
		assertEquals(StepId.APOTHECARY, GeneratorTestSupport.findLabelled(build(),
			StepId.APOTHECARY, IStepLabel.APOTHECARY_ATTACKER).getId());
	}

	// Rust: foul_catch_scatter_labelled_end_fouling
	@Test
	public void foulCatchScatterLabelledEndFouling() {
		assertNotNull(GeneratorTestSupport.findLabelled(build(),
			StepId.CATCH_SCATTER_THROW_IN, IStepLabel.END_FOULING));
	}
}
