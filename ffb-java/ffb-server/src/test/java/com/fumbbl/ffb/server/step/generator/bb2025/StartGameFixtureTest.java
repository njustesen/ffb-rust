package com.fumbbl.ffb.server.step.generator.bb2025;

import com.fumbbl.ffb.server.GameState;
import com.fumbbl.ffb.server.fixture.GameFixture;
import com.fumbbl.ffb.server.step.IStep;
import com.fumbbl.ffb.server.step.StepId;
import com.fumbbl.ffb.server.step.generator.GeneratorTestSupport;
import com.fumbbl.ffb.server.step.generator.SequenceGenerator;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirrors the Rust push-order tests in
 * {@code ffb-rust/crates/ffb-engine/src/step/generator/bb2025/start_game.rs}.
 */
public class StartGameFixtureTest {

	private GameState gameState;

	@BeforeEach
	public void setUp() {
		gameState = GameFixture.createGameState(3);
	}

	private IStep[] build() {
		new StartGame().pushSequence(new SequenceGenerator.SequenceParams(gameState));
		return GeneratorTestSupport.sequence(gameState);
	}

	// Rust: start_game_sequence_has_five_steps
	@Test
	public void startGameSequenceHasFiveSteps() {
		assertEquals(5, build().length);
	}

	// Rust: start_game_sequence_starts_with_init_start_game
	@Test
	public void startGameSequenceStartsWithInitStartGame() {
		assertEquals(StepId.INIT_START_GAME, build()[0].getId());
	}

	// Rust: start_game_sequence_ends_with_buy_inducements
	@Test
	public void startGameSequenceEndsWithBuyInducements() {
		IStep[] steps = build();
		assertEquals(StepId.BUY_INDUCEMENTS, steps[steps.length - 1].getId());
	}

	// Rust: start_game_has_weather_and_spectators
	@Test
	public void startGameHasWeatherAndSpectators() {
		IStep[] steps = build();
		assertTrue(GeneratorTestSupport.contains(steps, StepId.WEATHER));
		assertTrue(GeneratorTestSupport.contains(steps, StepId.SPECTATORS));
	}

	// Rust: start_game_has_petty_cash
	@Test
	public void startGameHasPettyCash() {
		assertTrue(GeneratorTestSupport.contains(build(), StepId.PETTY_CASH));
	}
}
