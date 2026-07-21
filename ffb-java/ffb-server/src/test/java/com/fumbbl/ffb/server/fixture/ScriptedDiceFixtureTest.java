package com.fumbbl.ffb.server.fixture;

import com.fumbbl.ffb.Weather;
import com.fumbbl.ffb.model.Game;
import com.fumbbl.ffb.server.GameState;
import com.fumbbl.ffb.server.step.IStep;
import com.fumbbl.ffb.server.step.StepAction;
import com.fumbbl.ffb.server.step.StepId;

import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

/**
 * Self-test for {@link GameFixture#installScriptedDice(GameState, int...)}.
 *
 * <p>Proves that a known scripted roll sequence drives a real rolling step to a
 * known outcome. {@code StepWeather} rolls 2d6 via
 * {@code gameState.getDiceRoller().rollWeather()} and maps the total to the
 * weather table (2=SWELTERING_HEAT, 3=VERY_SUNNY, 4-10=NICE, 11=POURING_RAIN,
 * 12=BLIZZARD), so the two scripted faces fully determine the result.
 */
public class ScriptedDiceFixtureTest {

	private GameState newState() {
		return GameFixture.createGameState(2);
	}

	private Weather runWeatherStep(GameState gameState) {
		Game game = gameState.getGame();
		IStep step = GameFixture.createStep(gameState, StepId.WEATHER);
		StepAction action = GameFixture.startStep(step);
		// StepWeather rolls in start() and asks the loop to advance.
		assertEquals(StepAction.NEXT_STEP, action);
		return game.getFieldModel().getWeather();
	}

	@Test
	public void scriptedElevenGivesPouringRain() {
		GameState gameState = newState();
		GameFixture.installScriptedDice(gameState, 6, 5); // 2d6 = 11
		assertEquals(Weather.POURING_RAIN, runWeatherStep(gameState));
	}

	@Test
	public void scriptedSnakeEyesGivesSwelteringHeat() {
		GameState gameState = newState();
		GameFixture.installScriptedDice(gameState, 1, 1); // 2d6 = 2
		assertEquals(Weather.SWELTERING_HEAT, runWeatherStep(gameState));
	}

	@Test
	public void scriptedBoxcarsGivesBlizzard() {
		GameState gameState = newState();
		GameFixture.installScriptedDice(gameState, 6, 6); // 2d6 = 12
		assertEquals(Weather.BLIZZARD, runWeatherStep(gameState));
	}

	@Test
	public void rawD6DrawsConsumeScriptInOrder() {
		GameState gameState = newState();
		GameFixture.installScriptedDice(gameState, 3, 4, 1);
		assertEquals(3, gameState.getDiceRoller().rollDice(6));
		assertEquals(4, gameState.getDiceRoller().rollDice(6));
		assertEquals(1, gameState.getDiceRoller().rollDice(6));
	}

	@Test
	public void blockDiceUseScriptedFaces() {
		GameState gameState = newState();
		// Block dice are raw d6 faces; 6 = POW, 1 = skull.
		GameFixture.installScriptedDice(gameState, 6, 1);
		int[] blockRoll = gameState.getDiceRoller().rollBlockDice(2);
		assertEquals(6, blockRoll[0]);
		assertEquals(1, blockRoll[1]);
	}

	@Test
	public void exhaustedScriptFallsBackToRandom() {
		GameState gameState = newState();
		GameFixture.installScriptedDice(gameState, 4);
		assertEquals(4, gameState.getDiceRoller().rollDice(6));
		// No exception, still a valid die face, just no longer deterministic.
		int fallback = gameState.getDiceRoller().rollDice(6);
		org.junit.jupiter.api.Assertions.assertTrue(fallback >= 1 && fallback <= 6);
	}
}
