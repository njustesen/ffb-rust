package com.fumbbl.ffb.server.step.generator.bb2025;

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
 * {@code ffb-rust/crates/ffb-engine/src/step/generator/bb2025/punt.rs}.
 */
public class PuntFixtureTest {

	private GameState gameState;

	@BeforeEach
	public void setUp() {
		gameState = GameFixture.createGameState(3);
	}

	private IStep[] build() {
		new Punt().pushSequence(new SequenceGenerator.SequenceParams(gameState));
		return GeneratorTestSupport.sequence(gameState);
	}

	// Rust: punt_has_18_steps_with_activation
	@Test
	public void puntHas18StepsWithActivation() {
		IStep[] steps = build();
		assertEquals(18, steps.length);
		assertEquals(StepId.INIT_ACTIVATION, steps[1].getId());
	}

	// Rust: punt_ends_with_end_punt_labelled_end
	@Test
	public void puntEndsWithEndPuntLabelledEnd() {
		IStep[] steps = build();
		IStep last = steps[steps.length - 1];
		assertEquals(StepId.END_PUNT, last.getId());
		assertEquals(IStepLabel.END, last.getLabel());
	}

	// Rust: punt_catch_scatter_is_labelled_scatter_ball
	@Test
	public void puntCatchScatterIsLabelledScatterBall() {
		assertNotNull(GeneratorTestSupport.findLabelled(build(),
			StepId.CATCH_SCATTER_THROW_IN, IStepLabel.SCATTER_BALL));
	}

	// Rust: punt_starts_with_init_punt
	@Test
	public void puntStartsWithInitPunt() {
		assertEquals(StepId.INIT_PUNT, build()[0].getId());
	}

	// Rust: punt_has_punt_direction_and_punt_distance
	@Test
	public void puntHasPuntDirectionAndPuntDistance() {
		IStep[] steps = build();
		assertTrue(GeneratorTestSupport.contains(steps, StepId.PUNT_DIRECTION));
		assertTrue(GeneratorTestSupport.contains(steps, StepId.PUNT_DISTANCE));
	}
}
