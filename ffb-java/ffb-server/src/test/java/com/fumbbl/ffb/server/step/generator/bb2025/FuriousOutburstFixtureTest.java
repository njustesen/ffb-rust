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
 * {@code ffb-rust/crates/ffb-engine/src/step/generator/bb2025/furious_outburst.rs}.
 */
public class FuriousOutburstFixtureTest {

	private GameState gameState;

	@BeforeEach
	public void setUp() {
		gameState = GameFixture.createGameState(3);
	}

	private IStep[] build() {
		new FuriousOutburst().pushSequence(new SequenceGenerator.SequenceParams(gameState));
		return GeneratorTestSupport.sequence(gameState);
	}

	// Rust: furious_outburst_has_28_steps_with_activation
	@Test
	public void furiousOutburstHas28StepsWithActivation() {
		IStep[] steps = build();
		assertEquals(28, steps.length);
		assertEquals(StepId.INIT_ACTIVATION, steps[1].getId());
	}

	// Rust: furious_outburst_ends_with_end_furious_outburst_labelled_end
	@Test
	public void furiousOutburstEndsWithEndFuriousOutburstLabelledEnd() {
		IStep[] steps = build();
		IStep last = steps[steps.length - 1];
		assertEquals(StepId.END_FURIOUS_OUTBURST, last.getId());
		assertEquals(IStepLabel.END, last.getLabel());
	}

	// Rust: furious_outburst_place_ball_is_labelled_next
	@Test
	public void furiousOutburstPlaceBallIsLabelledNext() {
		assertNotNull(GeneratorTestSupport.findLabelled(build(), StepId.PLACE_BALL, IStepLabel.NEXT));
	}

	// Rust: furious_outburst_starts_with_init_furious_outburst
	@Test
	public void furiousOutburstStartsWithInitFuriousOutburst() {
		assertEquals(StepId.INIT_FURIOUS_OUTBURST, build()[0].getId());
	}

	// Rust: furious_outburst_has_stab
	@Test
	public void furiousOutburstHasStab() {
		assertTrue(GeneratorTestSupport.contains(build(), StepId.STAB));
	}
}
