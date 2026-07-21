package com.fumbbl.ffb.server.step.game.start;

import com.fumbbl.ffb.Weather;
import com.fumbbl.ffb.model.Game;
import com.fumbbl.ffb.report.ReportId;
import com.fumbbl.ffb.server.GameState;
import com.fumbbl.ffb.server.fixture.GameFixture;
import com.fumbbl.ffb.server.step.StepAction;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import java.util.EnumSet;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirrors the Rust unit tests in
 * {@code ffb-rust/crates/ffb-engine/src/step/game/start/step_weather.rs}
 * ({@code StepWeather}).
 *
 * <p>The step rolls 2d6, maps it to a {@link Weather}, sets it on the field
 * model, adds a {@code ReportWeather}, then advances. Rust seeds its RNG for
 * determinism; the Java fixture uses the real {@code DiceRoller}, so tests that
 * relied on a seed either assert over all valid outcomes or repeat across many
 * rolls (mirroring the multi-seed Rust tests).
 */
public class StepWeatherFixtureTest {

	private static final EnumSet<Weather> VALID_WEATHER = EnumSet.of(
		Weather.SWELTERING_HEAT, Weather.VERY_SUNNY, Weather.NICE,
		Weather.POURING_RAIN, Weather.BLIZZARD);

	private GameState gameState;
	private Game game;
	private StepWeather step;

	@BeforeEach
	public void setUp() {
		gameState = GameFixture.createGameState(3);
		game = gameState.getGame();
		step = new StepWeather(gameState);
	}

	// Rust: start_sets_weather_on_field_model
	@Test
	public void startSetsWeatherOnFieldModel() {
		StepAction action = GameFixture.startStep(step);
		assertEquals(StepAction.NEXT_STEP, action);
		assertTrue(VALID_WEATHER.contains(game.getFieldModel().getWeather()));
	}

	// Rust: start_returns_next_step
	@Test
	public void startReturnsNextStep() {
		assertEquals(StepAction.NEXT_STEP, GameFixture.startStep(step));
	}

	// Rust: weather_changes_from_default
	@Test
	public void weatherChangesFromDefault() {
		// Fixture default weather is NICE; over many real rolls at least one must differ.
		boolean changed = false;
		for (int i = 0; i < 100 && !changed; i++) {
			GameState gs = GameFixture.createGameState(3);
			Weather before = gs.getGame().getFieldModel().getWeather();
			new StepWeather(gs).start();
			if (gs.getGame().getFieldModel().getWeather() != before) {
				changed = true;
			}
		}
		assertTrue(changed, "Expected at least one roll to produce non-NICE weather");
	}

	// Rust: start_adds_weather_report
	@Test
	public void startAddsWeatherReport() {
		step.start();
		assertTrue(step.getResult().getReportList().hasReport(ReportId.WEATHER));
	}

	// Rust: weather_report_added_for_multiple_seeds
	@Test
	public void weatherReportAddedForMultipleRolls() {
		for (int i = 0; i < 5; i++) {
			GameState gs = GameFixture.createGameState(3);
			StepWeather s = new StepWeather(gs);
			s.start();
			assertTrue(s.getResult().getReportList().hasReport(ReportId.WEATHER),
				"iteration " + i + " should add a weather report");
		}
	}
}
