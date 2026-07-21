package com.fumbbl.ffb.server.step.generator.bb2016;

import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.server.GameState;
import com.fumbbl.ffb.server.fixture.GameFixture;
import com.fumbbl.ffb.server.fixture.GeneratorTestSupport;
import com.fumbbl.ffb.server.step.IStep;
import com.fumbbl.ffb.server.step.StepId;
import com.fumbbl.ffb.server.step.generator.SequenceGenerator;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirrors the Rust push-order tests in
 * {@code ffb-rust/crates/ffb-engine/src/step/generator/bb2016/start_game.rs}.
 */
public class StartGameFixtureTest {

	private GameState gameState;

	@BeforeEach
	public void setUp() {
		gameState = GameFixture.createGameState(3, RulesCollection.Rules.BB2016);
	}

	private IStep[] build() {
		new StartGame().pushSequence(new SequenceGenerator.SequenceParams(gameState));
		return GeneratorTestSupport.sequence(gameState);
	}

	// Rust: start_game_starts_with_init_start_game
	@Test
	public void startGameStartsWithInitStartGame() {
		assertEquals(StepId.INIT_START_GAME, build()[0].getId());
	}

	// Rust: start_game_ends_with_spectators
	@Test
	public void startGameEndsWithSpectators() {
		IStep[] steps = build();
		assertEquals(StepId.SPECTATORS, steps[steps.length - 1].getId());
	}

	// Rust: start_game_has_6_steps
	@Test
	public void startGameHas6Steps() {
		assertEquals(6, build().length);
	}

	// Rust: start_game_contains_weather_petty_cash_buy_cards_buy_inducements
	@Test
	public void startGameContainsWeatherPettyCashBuyCardsBuyInducements() {
		IStep[] steps = build();
		assertTrue(GeneratorTestSupport.contains(steps, StepId.WEATHER));
		assertTrue(GeneratorTestSupport.contains(steps, StepId.PETTY_CASH));
		assertTrue(GeneratorTestSupport.contains(steps, StepId.BUY_CARDS));
		assertTrue(GeneratorTestSupport.contains(steps, StepId.BUY_INDUCEMENTS));
	}

	// Rust: build_sequence_returns_vec
	@Test
	public void buildSequenceReturnsVec() {
		assertTrue(build().length > 0);
	}
}
