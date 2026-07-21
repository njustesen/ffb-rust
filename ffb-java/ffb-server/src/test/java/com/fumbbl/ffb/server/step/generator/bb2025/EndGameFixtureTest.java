package com.fumbbl.ffb.server.step.generator.bb2025;

import com.fumbbl.ffb.server.GameState;
import com.fumbbl.ffb.server.fixture.GameFixture;
import com.fumbbl.ffb.server.step.IStep;
import com.fumbbl.ffb.server.step.IStepLabel;
import com.fumbbl.ffb.server.step.StepId;
import com.fumbbl.ffb.server.step.generator.GeneratorTestSupport;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNotNull;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirrors the Rust push-order tests in
 * {@code ffb-rust/crates/ffb-engine/src/step/generator/bb2025/end_game.rs}.
 */
public class EndGameFixtureTest {

	private GameState gameState;

	@BeforeEach
	public void setUp() {
		gameState = GameFixture.createGameState(3);
	}

	private IStep[] build(boolean adminMode) {
		new EndGame().pushSequence(new EndGame.SequenceParams(gameState, adminMode));
		return GeneratorTestSupport.sequence(gameState);
	}

	// Rust: end_game_has_7_steps
	@Test
	public void endGameHas7Steps() {
		assertEquals(7, build(false).length);
	}

	// Rust: end_game_ends_with_end_game_labelled_end_game
	@Test
	public void endGameEndsWithEndGameLabelledEndGame() {
		IStep[] steps = build(false);
		IStep last = steps[steps.length - 1];
		assertEquals(StepId.END_GAME, last.getId());
		assertEquals(IStepLabel.END_GAME, last.getLabel());
	}

	// Rust: first_step_is_init_end_game
	@Test
	public void firstStepIsInitEndGame() {
		assertEquals(StepId.INIT_END_GAME, build(false)[0].getId());
	}

	// Rust: admin_mode_param_set_in_first_step
	@Test
	public void adminModeParamSetInFirstStep() {
		IStep init = build(true)[0];
		assertTrue(GeneratorTestSupport.booleanField(init, "fAdminMode"));
	}

	// Rust: contains_mvp_step
	@Test
	public void containsMvpStep() {
		assertTrue(GeneratorTestSupport.contains(build(false), StepId.MVP));
	}

	// Rust: contains_winnings_step
	@Test
	public void containsWinningsStep() {
		assertNotNull(GeneratorTestSupport.find(build(false), StepId.WINNINGS));
	}
}
