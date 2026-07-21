package com.fumbbl.ffb.server.step.generator.bb2025;

import com.fumbbl.ffb.FieldCoordinate;
import com.fumbbl.ffb.PlayerState;
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
 * {@code ffb-rust/crates/ffb-engine/src/step/generator/bb2025/scatter_player.rs}.
 */
public class ScatterPlayerFixtureTest {

	private GameState gameState;

	@BeforeEach
	public void setUp() {
		gameState = GameFixture.createGameState(3);
	}

	private IStep[] build(boolean hasSwoop) {
		new ScatterPlayer().pushSequence(
			new ScatterPlayer.SequenceParams(gameState, "p1", new PlayerState(PlayerState.STANDING),
				false, new FieldCoordinate(1, 1), hasSwoop, false));
		return GeneratorTestSupport.sequence(gameState);
	}

	// Rust: scatter_player_without_swoop_has_8_steps
	@Test
	public void scatterPlayerWithoutSwoopHas8Steps() {
		assertEquals(8, build(false).length);
	}

	// Rust: scatter_player_with_swoop_has_9_steps
	@Test
	public void scatterPlayerWithSwoopHas9Steps() {
		assertEquals(9, build(true).length);
	}

	// Rust: scatter_player_ends_with_end_scatter_player_labelled_end
	@Test
	public void scatterPlayerEndsWithEndScatterPlayerLabelledEnd() {
		IStep[] steps = build(false);
		IStep last = steps[steps.length - 1];
		assertEquals(StepId.END_SCATTER_PLAYER, last.getId());
		assertEquals(IStepLabel.END, last.getLabel());
	}

	// Rust: scatter_player_apothecary_hit_player_is_labelled
	@Test
	public void scatterPlayerApothecaryHitPlayerIsLabelled() {
		assertNotNull(GeneratorTestSupport.findLabelled(build(false),
			StepId.APOTHECARY, IStepLabel.APOTHECARY_HIT_PLAYER));
	}

	// Rust: build_sequence_returns_vec
	@Test
	public void buildSequenceReturnsVec() {
		assertTrue(build(false).length > 0);
	}
}
