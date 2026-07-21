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
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirrors the Rust push-order tests in
 * {@code ffb-rust/crates/ffb-engine/src/step/generator/bb2020/end_game.rs}.
 * Uses a BB2020 GameState so the edition-specific end-game steps
 * (ASSIGN_TOUCHDOWNS, PENALTY_SHOOTOUT) can be created.
 */
public class EndGameFixtureTest {

	private GameState gameState;

	@BeforeEach
	public void setUp() {
		gameState = GameFixture.createGameState(3, RulesCollection.Rules.BB2020);
	}

	private IStep[] build(boolean adminMode) {
		new EndGame().pushSequence(new EndGame.SequenceParams(gameState, adminMode));
		return GeneratorTestSupport.sequence(gameState);
	}

	// Rust: end_game_has_8_steps
	@Test
	public void endGameHas8Steps() {
		assertEquals(8, build(false).length);
	}

	// Rust: end_game_ends_with_end_game_labelled
	@Test
	public void endGameEndsWithEndGameLabelled() {
		IStep[] steps = build(false);
		IStep last = steps[steps.length - 1];
		assertEquals(StepId.END_GAME, last.getId());
		assertEquals(IStepLabel.END_GAME, last.getLabel());
	}

	// Rust: end_game_has_assign_touchdowns
	@Test
	public void endGameHasAssignTouchdowns() {
		assertTrue(GeneratorTestSupport.contains(build(false), StepId.ASSIGN_TOUCHDOWNS));
	}

	// Rust: end_game_has_penalty_shootout
	@Test
	public void endGameHasPenaltyShootout() {
		assertTrue(GeneratorTestSupport.contains(build(false), StepId.PENALTY_SHOOTOUT));
	}

	// Rust: admin_mode_param_passed_to_init
	@Test
	public void adminModeParamPassedToInit() {
		assertTrue(GeneratorTestSupport.booleanField(build(true)[0], "fAdminMode"));
	}

	// Rust: end_game_starts_with_init_end_game
	@Test
	public void endGameStartsWithInitEndGame() {
		assertEquals(StepId.INIT_END_GAME, build(false)[0].getId());
	}
}
