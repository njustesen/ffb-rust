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
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirrors the Rust push-order tests in
 * {@code ffb-rust/crates/ffb-engine/src/step/generator/bb2025/throw_keg.rs}.
 *
 * Rust's {@code no_player_id_produces_empty_throw_keg_params} is intentionally not
 * mirrored: Java's pushSequence always threads {@code from(TARGET_PLAYER_ID, ...)}
 * (even for a null id), whereas the Rust generator omits the param when None.
 */
public class ThrowKegFixtureTest {

	private GameState gameState;

	@BeforeEach
	public void setUp() {
		gameState = GameFixture.createGameState(3);
	}

	private IStep[] build(String playerId) {
		new ThrowKeg().pushSequence(new ThrowKeg.SequenceParams(gameState, playerId));
		return GeneratorTestSupport.sequence(gameState);
	}

	// Rust: throw_keg_has_19_steps_with_activation
	@Test
	public void throwKegHas19StepsWithActivation() {
		IStep[] steps = build(null);
		assertEquals(19, steps.length);
		assertEquals(StepId.INIT_ACTIVATION, steps[0].getId());
	}

	// Rust: throw_keg_ends_with_end_throw_keg_labelled_end
	@Test
	public void throwKegEndsWithEndThrowKegLabelledEnd() {
		IStep[] steps = build(null);
		IStep last = steps[steps.length - 1];
		assertEquals(StepId.END_THROW_KEG, last.getId());
		assertEquals(IStepLabel.END, last.getLabel());
	}

	// Rust: throw_keg_step_follows_activation_sub_sequence
	@Test
	public void throwKegStepFollowsActivationSubSequence() {
		assertEquals(StepId.THROW_KEG, build(null)[13].getId());
	}

	// Rust: player_id_param_wired_when_provided
	@Test
	public void playerIdParamWiredWhenProvided() {
		IStep throwKeg = build("p1")[13];
		assertEquals("p1", GeneratorTestSupport.readField(throwKeg, "playerId"));
	}

	// Rust: contains_apothecary_step
	@Test
	public void containsApothecaryStep() {
		assertTrue(GeneratorTestSupport.contains(build(null), StepId.APOTHECARY));
	}
}
