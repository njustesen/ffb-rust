package com.fumbbl.ffb.server.step.generator.bb2020;

import com.fumbbl.ffb.FieldCoordinate;
import com.fumbbl.ffb.PlayerState;
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
import static org.junit.jupiter.api.Assertions.assertNotNull;
import static org.junit.jupiter.api.Assertions.assertNull;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirrors the Rust push-order tests in
 * {@code ffb-rust/crates/ffb-engine/src/step/generator/bb2020/scatter_player.rs}.
 */
public class ScatterPlayerFixtureTest {

	private GameState gameState;

	@BeforeEach
	public void setUp() {
		gameState = GameFixture.createGameState(3, RulesCollection.Rules.BB2020);
	}

	private IStep[] build(boolean hasSwoop, boolean deviate, boolean crashLanding) {
		new ScatterPlayer().pushSequence(new ScatterPlayer.SequenceParams(gameState, "p1",
			new PlayerState(PlayerState.STANDING), false, new FieldCoordinate(1, 1), hasSwoop, false,
			deviate, crashLanding));
		return GeneratorTestSupport.sequence(gameState);
	}

	// Rust: scatter_player_without_swoop_has_7_steps
	@Test
	public void scatterPlayerWithoutSwoopHas7Steps() {
		assertEquals(7, build(false, false, false).length);
	}

	// Rust: scatter_player_with_swoop_has_8_steps
	@Test
	public void scatterPlayerWithSwoopHas8Steps() {
		assertEquals(8, build(true, false, false).length);
	}

	// Rust: scatter_player_ends_with_end_scatter_player_no_label
	@Test
	public void scatterPlayerEndsWithEndScatterPlayerNoLabel() {
		IStep[] steps = build(false, false, false);
		IStep last = steps[steps.length - 1];
		assertEquals(StepId.END_SCATTER_PLAYER, last.getId());
		assertNull(last.getLabel());
	}

	// Rust: scatter_player_has_no_steady_footing
	@Test
	public void scatterPlayerHasNoSteadyFooting() {
		assertFalse(GeneratorTestSupport.contains(build(false, false, false), StepId.STEADY_FOOTING));
	}

	// Rust: scatter_player_apothecary_hit_player_is_labelled
	@Test
	public void scatterPlayerApothecaryHitPlayerIsLabelled() {
		assertNotNull(GeneratorTestSupport.findLabelled(build(false, false, false),
			StepId.APOTHECARY, IStepLabel.APOTHECARY_HIT_PLAYER));
	}

	// Rust: scatter_player_init_has_pass_deviates_and_crash_landing
	@Test
	public void scatterPlayerInitHasPassDeviatesAndCrashLanding() {
		IStep init = GeneratorTestSupport.find(build(false, true, true), StepId.INIT_SCATTER_PLAYER);
		assertTrue(GeneratorTestSupport.booleanField(init, "deviate"));
		assertTrue(GeneratorTestSupport.booleanField(init, "crashLanding"));
	}
}
