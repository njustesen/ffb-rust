package com.fumbbl.ffb.server.step.generator.bb2020;

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
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirrors the Rust push-order tests in
 * {@code ffb-rust/crates/ffb-engine/src/step/generator/bb2020/start_game.rs}.
 */
public class StartGameFixtureTest {

	private GameState gameState;

	@BeforeEach
	public void setUp() {
		gameState = GameFixture.createGameState(3, RulesCollection.Rules.BB2020);
	}

	private IStep[] build() {
		new StartGame().pushSequence(new SequenceGenerator.SequenceParams(gameState));
		return GeneratorTestSupport.sequence(gameState);
	}

	// Rust: start_game_has_five_steps
	@Test
	public void startGameHasFiveSteps() {
		assertEquals(5, build().length);
	}

	// Rust: start_game_starts_with_init_start_game
	@Test
	public void startGameStartsWithInitStartGame() {
		assertEquals(StepId.INIT_START_GAME, build()[0].getId());
	}

	// Rust: start_game_ends_with_buy_cards_and_inducements
	@Test
	public void startGameEndsWithBuyCardsAndInducements() {
		IStep[] steps = build();
		assertEquals(StepId.BUY_CARDS_AND_INDUCEMENTS, steps[steps.length - 1].getId());
	}

	// Rust: start_game_has_petty_cash
	@Test
	public void startGameHasPettyCash() {
		assertTrue(GeneratorTestSupport.contains(build(), StepId.PETTY_CASH));
	}

	// Rust: start_game_has_weather
	@Test
	public void startGameHasWeather() {
		assertTrue(GeneratorTestSupport.contains(build(), StepId.WEATHER));
	}

	// Rust: start_game_has_buy_cards_and_inducements_not_buy_inducements
	@Test
	public void startGameHasBuyCardsAndInducementsNotBuyInducements() {
		IStep[] steps = build();
		assertTrue(GeneratorTestSupport.contains(steps, StepId.BUY_CARDS_AND_INDUCEMENTS));
		assertFalse(GeneratorTestSupport.contains(steps, StepId.BUY_INDUCEMENTS));
	}
}
