package com.fumbbl.ffb.server.step.generator.bb2020;

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
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertNull;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirrors the Rust push-order tests in
 * {@code ffb-rust/crates/ffb-engine/src/step/generator/bb2020/throw_keg.rs}.
 *
 * Rust's {@code player_id_absent_when_none} is not mirrored: Java always threads
 * the TARGET_PLAYER_ID param (even for a null id).
 */
public class ThrowKegFixtureTest {

	private GameState gameState;

	@BeforeEach
	public void setUp() {
		gameState = GameFixture.createGameState(3, RulesCollection.Rules.BB2020);
	}

	private IStep[] build(String playerId) {
		new ThrowKeg().pushSequence(new ThrowKeg.SequenceParams(gameState, playerId));
		return GeneratorTestSupport.sequence(gameState);
	}

	// Rust: throw_keg_has_activation_block
	@Test
	public void throwKegHasActivationBlock() {
		assertTrue(GeneratorTestSupport.contains(build(null), StepId.INIT_ACTIVATION));
	}

	// Rust: throw_keg_ends_with_end_throw_keg_labelled_end
	@Test
	public void throwKegEndsWithEndThrowKegLabelledEnd() {
		IStep[] steps = build(null);
		IStep last = steps[steps.length - 1];
		assertEquals(StepId.END_THROW_KEG, last.getId());
		assertEquals(IStepLabel.END, last.getLabel());
	}

	// Rust: throw_keg_bone_head_has_no_label
	@Test
	public void throwKegBoneHeadHasNoLabel() {
		assertNull(GeneratorTestSupport.find(build(null), StepId.BONE_HEAD).getLabel());
	}

	// Rust: throw_keg_has_no_steady_footing
	@Test
	public void throwKegHasNoSteadyFooting() {
		assertFalse(GeneratorTestSupport.contains(build(null), StepId.STEADY_FOOTING));
	}

	// Rust: player_id_passed_when_some
	@Test
	public void playerIdPassedWhenSome() {
		IStep keg = GeneratorTestSupport.find(build("target_player"), StepId.THROW_KEG);
		assertEquals("target_player", GeneratorTestSupport.readField(keg, "playerId"));
	}
}
