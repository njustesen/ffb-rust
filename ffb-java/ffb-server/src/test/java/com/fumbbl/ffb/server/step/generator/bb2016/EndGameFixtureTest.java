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
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirrors the Rust push-order tests in
 * {@code ffb-rust/crates/ffb-engine/src/step/generator/bb2016/end_game.rs}.
 */
public class EndGameFixtureTest {

	private GameState gameState;

	@BeforeEach
	public void setUp() {
		gameState = GameFixture.createGameState(3, RulesCollection.Rules.BB2016);
	}

	private IStep[] build(boolean adminMode) {
		new EndGame().pushSequence(new EndGame.SequenceParams(gameState, adminMode));
		return GeneratorTestSupport.sequence(gameState);
	}

	// Rust: end_game_starts_with_init_end_game
	@Test
	public void endGameStartsWithInitEndGame() {
		assertEquals(StepId.INIT_END_GAME, build(false)[0].getId());
	}

	// Rust: end_game_ends_with_end_game_labelled
	@Test
	public void endGameEndsWithEndGameLabelled() {
		IStep[] steps = build(false);
		IStep last = steps[steps.length - 1];
		assertEquals(StepId.END_GAME, last.getId());
		assertEquals(IStepLabel.END_GAME, last.getLabel());
	}

	// Rust: end_game_has_mvp_and_winnings
	@Test
	public void endGameHasMvpAndWinnings() {
		IStep[] steps = build(false);
		assertTrue(GeneratorTestSupport.contains(steps, StepId.MVP));
		assertTrue(GeneratorTestSupport.contains(steps, StepId.WINNINGS));
	}

	// Rust: end_game_has_penalty_shootout_and_fan_factor
	@Test
	public void endGameHasPenaltyShootoutAndFanFactor() {
		IStep[] steps = build(false);
		assertTrue(GeneratorTestSupport.contains(steps, StepId.PENALTY_SHOOTOUT));
		assertTrue(GeneratorTestSupport.contains(steps, StepId.FAN_FACTOR));
	}

	// Rust: admin_mode_param_passed_to_init
	@Test
	public void adminModeParamPassedToInit() {
		assertTrue(GeneratorTestSupport.booleanField(build(true)[0], "fAdminMode"));
	}

	// Rust: end_game_has_seven_steps
	@Test
	public void endGameHasSevenSteps() {
		assertEquals(7, build(false).length);
	}
}
