package com.fumbbl.ffb.server.step.generator.bb2016;

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
import static org.junit.jupiter.api.Assertions.assertNotNull;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirrors the Rust push-order tests in
 * {@code ffb-rust/crates/ffb-engine/src/step/generator/bb2016/blitz_move.rs}.
 */
public class BlitzMoveFixtureTest {

	private GameState gameState;

	@BeforeEach
	public void setUp() {
		gameState = GameFixture.createGameState(3, RulesCollection.Rules.BB2016);
	}

	private IStep[] build() {
		new BlitzMove().pushSequence(new BlitzMove.SequenceParams(gameState));
		return GeneratorTestSupport.sequence(gameState);
	}

	// Rust: blitz_move_starts_with_init_moving
	@Test
	public void blitzMoveStartsWithInitMoving() {
		assertEquals(StepId.INIT_MOVING, build()[0].getId());
	}

	// Rust: blitz_move_ends_with_end_moving
	@Test
	public void blitzMoveEndsWithEndMoving() {
		IStep[] steps = build();
		IStep last = steps[steps.length - 1];
		assertEquals(StepId.END_MOVING, last.getId());
		assertEquals(IStepLabel.END_MOVING, last.getLabel());
	}

	// Rust: blitz_move_contains_bone_head
	@Test
	public void blitzMoveContainsBoneHead() {
		assertTrue(GeneratorTestSupport.contains(build(), StepId.BONE_HEAD));
	}

	// Rust: blitz_move_shadowing_is_labelled
	@Test
	public void blitzMoveShadowingIsLabelled() {
		assertEquals(IStepLabel.SHADOWING,
			GeneratorTestSupport.findLabelled(build(), StepId.SHADOWING, IStepLabel.SHADOWING).getLabel());
	}

	// Rust: blitz_move_scatter_ball_is_labelled
	@Test
	public void blitzMoveScatterBallIsLabelled() {
		assertNotNull(GeneratorTestSupport.findLabelled(build(),
			StepId.CATCH_SCATTER_THROW_IN, IStepLabel.SCATTER_BALL));
	}
}
