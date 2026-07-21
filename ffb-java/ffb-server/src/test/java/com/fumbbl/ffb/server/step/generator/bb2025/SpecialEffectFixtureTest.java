package com.fumbbl.ffb.server.step.generator.bb2025;

import com.fumbbl.ffb.ApothecaryMode;
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
 * {@code ffb-rust/crates/ffb-engine/src/step/generator/bb2025/special_effect.rs}.
 *
 * The generator class {@code SpecialEffect} shares its simple name with the enum
 * {@code com.fumbbl.ffb.SpecialEffect}; the enum is referenced fully-qualified.
 */
public class SpecialEffectFixtureTest {

	private GameState gameState;

	@BeforeEach
	public void setUp() {
		gameState = GameFixture.createGameState(3);
	}

	private IStep[] build(com.fumbbl.ffb.SpecialEffect effect, String playerId, boolean rollForEffect) {
		new SpecialEffect().pushSequence(new SpecialEffect.SequenceParams(gameState, effect, playerId, rollForEffect));
		return GeneratorTestSupport.sequence(gameState);
	}

	// Rust: special_effect_has_5_steps
	@Test
	public void specialEffectHas5Steps() {
		assertEquals(5, build(com.fumbbl.ffb.SpecialEffect.LIGHTNING, "p1", false).length);
	}

	// Rust: special_effect_next_step_is_labelled_end_special_effect
	@Test
	public void specialEffectNextStepIsLabelledEndSpecialEffect() {
		IStep[] steps = build(com.fumbbl.ffb.SpecialEffect.LIGHTNING, "p1", false);
		IStep last = steps[steps.length - 1];
		assertEquals(StepId.NEXT_STEP, last.getId());
		assertEquals(IStepLabel.END_SPECIAL_EFFECT, last.getLabel());
	}

	// Rust: special_effect_key_passed_to_first_step
	@Test
	public void specialEffectKeyPassedToFirstStep() {
		IStep[] steps = build(com.fumbbl.ffb.SpecialEffect.LIGHTNING, "p1", true);
		assertEquals(com.fumbbl.ffb.SpecialEffect.LIGHTNING,
			GeneratorTestSupport.readField(steps[0], "fSpecialEffect"));
	}

	// Rust: player_id_passed_to_first_step
	@Test
	public void playerIdPassedToFirstStep() {
		IStep[] steps = build(com.fumbbl.ffb.SpecialEffect.ZAP, "player42", false);
		assertEquals("player42", GeneratorTestSupport.readField(steps[0], "fPlayerId"));
	}

	// Rust: apothecary_step_uses_special_effect_mode
	@Test
	public void apothecaryStepUsesSpecialEffectMode() {
		IStep apo = GeneratorTestSupport.find(build(com.fumbbl.ffb.SpecialEffect.LIGHTNING, "p1", false), StepId.APOTHECARY);
		assertEquals(ApothecaryMode.SPECIAL_EFFECT, GeneratorTestSupport.readField(apo, "fApothecaryMode"));
	}
}
