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
import static org.junit.jupiter.api.Assertions.assertNotNull;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirrors the Rust push-order tests in
 * {@code ffb-rust/crates/ffb-engine/src/step/generator/bb2025/foul.rs}.
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

	// Rust: foul_sequence_starts_with_init_fouling
	@Test
	public void foulSequenceStartsWithInitFouling() {
		assertEquals(StepId.INIT_FOULING, build()[0].getId());
	}

	// Rust: foul_sequence_ends_with_end_fouling
	@Test
	public void foulSequenceEndsWithEndFouling() {
		IStep[] steps = build();
		assertEquals(StepId.END_FOULING, steps[steps.length - 1].getId());
	}

	// Rust: foul_catch_scatter_is_labelled_end_fouling
	@Test
	public void foulCatchScatterIsLabelledEndFouling() {
		assertNotNull(GeneratorTestSupport.findLabelled(build(),
			StepId.CATCH_SCATTER_THROW_IN, IStepLabel.END_FOULING));
	}

	// Rust: foul_apothecary_attacker_is_labelled
	@Test
	public void foulApothecaryAttackerIsLabelled() {
		assertNotNull(GeneratorTestSupport.findLabelled(build(),
			StepId.APOTHECARY, IStepLabel.APOTHECARY_ATTACKER));
	}

	// Rust: build_sequence_is_nonempty
	@Test
	public void buildSequenceIsNonempty() {
		assertTrue(build().length > 0);
	}
}
