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
import static org.junit.jupiter.api.Assertions.assertNull;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirrors the Rust push-order tests in
 * {@code ffb-rust/crates/ffb-engine/src/step/generator/bb2020/furious_outburst.rs}.
 */
public class FuriousOutburstFixtureTest {

	private GameState gameState;

	@BeforeEach
	public void setUp() {
		gameState = GameFixture.createGameState(3, RulesCollection.Rules.BB2020);
	}

	private IStep[] build() {
		new FuriousOutburst().pushSequence(new SequenceGenerator.SequenceParams(gameState));
		return GeneratorTestSupport.sequence(gameState);
	}

	// Rust: furious_outburst_ends_with_end_furious_outburst_labelled_end
	@Test
	public void furiousOutburstEndsWithEndFuriousOutburstLabelledEnd() {
		IStep[] steps = build();
		IStep last = steps[steps.length - 1];
		assertEquals(StepId.END_FURIOUS_OUTBURST, last.getId());
		assertEquals(IStepLabel.END, last.getLabel());
	}

	// Rust: furious_outburst_has_activation_block
	@Test
	public void furiousOutburstHasActivationBlock() {
		assertTrue(GeneratorTestSupport.contains(build(), StepId.INIT_ACTIVATION));
	}

	// Rust: furious_outburst_place_ball_is_labelled_next
	@Test
	public void furiousOutburstPlaceBallIsLabelledNext() {
		assertNotNull(GeneratorTestSupport.findLabelled(build(), StepId.PLACE_BALL, IStepLabel.NEXT));
	}

	// Rust: furious_outburst_blood_lust_has_no_failure_label
	@Test
	public void furiousOutburstBloodLustHasNoFailureLabel() {
		IStep bloodLust = GeneratorTestSupport.find(build(), StepId.BLOOD_LUST);
		Object state = GeneratorTestSupport.readField(bloodLust, "state");
		assertNull(GeneratorTestSupport.readField(state, "goToLabelOnFailure"));
	}

	// Rust: furious_outburst_starts_with_init_furious_outburst
	@Test
	public void furiousOutburstStartsWithInitFuriousOutburst() {
		assertEquals(StepId.INIT_FURIOUS_OUTBURST, build()[0].getId());
	}

	// Rust: furious_outburst_has_second_move
	@Test
	public void furiousOutburstHasSecondMove() {
		assertTrue(GeneratorTestSupport.contains(build(), StepId.SECOND_MOVE_FURIOUS_OUTBURST));
	}
}
