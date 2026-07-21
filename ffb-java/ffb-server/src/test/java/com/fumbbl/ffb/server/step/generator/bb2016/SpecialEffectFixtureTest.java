package com.fumbbl.ffb.server.step.generator.bb2016;

import com.fumbbl.ffb.ApothecaryMode;
import com.fumbbl.ffb.RulesCollection;
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
 * {@code ffb-rust/crates/ffb-engine/src/step/generator/bb2016/special_effect.rs}.
 * The generator class {@code SpecialEffect} shares its simple name with the enum
 * {@code com.fumbbl.ffb.SpecialEffect}; the enum is referenced fully-qualified.
 */
public class SpecialEffectFixtureTest {

	private GameState gameState;

	@BeforeEach
	public void setUp() {
		gameState = GameFixture.createGameState(3, RulesCollection.Rules.BB2016);
	}

	private IStep[] build(com.fumbbl.ffb.SpecialEffect effect, String playerId) {
		new SpecialEffect().pushSequence(new SpecialEffect.SequenceParams(gameState, effect, playerId, false));
		return GeneratorTestSupport.sequence(gameState);
	}

	// Rust: special_effect_starts_with_special_effect
	@Test
	public void specialEffectStartsWithSpecialEffect() {
		assertEquals(StepId.SPECIAL_EFFECT, build(com.fumbbl.ffb.SpecialEffect.FIREBALL, "p1")[0].getId());
	}

	// Rust: special_effect_ends_with_next_step_labelled
	@Test
	public void specialEffectEndsWithNextStepLabelled() {
		IStep[] steps = build(com.fumbbl.ffb.SpecialEffect.FIREBALL, "p1");
		IStep last = steps[steps.length - 1];
		assertEquals(StepId.NEXT_STEP, last.getId());
		assertEquals(IStepLabel.END_SPECIAL_EFFECT, last.getLabel());
	}

	// Rust: special_effect_has_3_steps
	@Test
	public void specialEffectHas3Steps() {
		assertEquals(3, build(com.fumbbl.ffb.SpecialEffect.FIREBALL, "p1").length);
	}

	// Rust: special_effect_apothecary_has_special_effect_mode
	@Test
	public void specialEffectApothecaryHasSpecialEffectMode() {
		IStep apo = GeneratorTestSupport.find(build(com.fumbbl.ffb.SpecialEffect.FIREBALL, "p1"), StepId.APOTHECARY);
		assertEquals(ApothecaryMode.SPECIAL_EFFECT, GeneratorTestSupport.readField(apo, "fApothecaryMode"));
	}

	// Rust: special_effect_key_passed_when_some
	@Test
	public void specialEffectKeyPassedWhenSome() {
		IStep[] steps = build(com.fumbbl.ffb.SpecialEffect.FIREBALL, "p1");
		assertEquals(com.fumbbl.ffb.SpecialEffect.FIREBALL,
			GeneratorTestSupport.readField(steps[0], "fSpecialEffect"));
	}

	// Rust: player_id_passed_when_some
	@Test
	public void playerIdPassedWhenSome() {
		IStep[] steps = build(com.fumbbl.ffb.SpecialEffect.ZAP, "pX");
		assertEquals("pX", GeneratorTestSupport.readField(steps[0], "fPlayerId"));
	}
}
