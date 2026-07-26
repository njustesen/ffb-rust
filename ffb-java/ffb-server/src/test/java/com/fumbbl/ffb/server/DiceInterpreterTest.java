package com.fumbbl.ffb.server;

import com.fumbbl.ffb.Weather;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirror of ffb-rust crates/ffb-engine/src/dice_interpreter.rs tests.
 * Rust exposes these as associated functions; Java's DiceInterpreter is a singleton (getInstance()).
 */
public class DiceInterpreterTest {

	private final DiceInterpreter di = DiceInterpreter.getInstance();

	// rust: interpret_roll_weather_sweltering_heat
	@Test
	public void interpretRollWeatherSwelteringHeat() {
		assertEquals(Weather.SWELTERING_HEAT, di.interpretRollWeather(new int[]{1, 1}));
	}

	// rust: interpret_weather_all_nice_values
	@Test
	public void interpretWeatherAllNiceValues() {
		for (int total = 4; total <= 10; total++) {
			assertEquals(Weather.NICE, di.interpretWeather(total), "total=" + total);
		}
	}

	// rust: roll_six_always_succeeds
	@Test
	public void rollSixAlwaysSucceeds() {
		assertTrue(di.isSkillRollSuccessful(6, 6));
		assertTrue(di.isSkillRollSuccessful(6, 7));
	}

	// rust: roll_one_always_fails
	@Test
	public void rollOneAlwaysFails() {
		assertFalse(di.isSkillRollSuccessful(1, 1));
		assertFalse(di.isSkillRollSuccessful(1, 2));
	}

	// rust: roll_meets_minimum
	@Test
	public void rollMeetsMinimum() {
		assertTrue(di.isSkillRollSuccessful(4, 4));
		assertFalse(di.isSkillRollSuccessful(3, 4));
	}

	// rust: minimum_roll_dauntless_capped_at_six
	@Test
	public void minimumRollDauntlessCappedAtSix() {
		assertEquals(6, di.minimumRollDauntless(1, 10));
		assertEquals(3, di.minimumRollDauntless(3, 5));
	}

	// rust: minimum_roll_confusion_good_conditions
	@Test
	public void minimumRollConfusionGoodConditions() {
		assertEquals(2, di.minimumRollConfusion(true));
		assertEquals(4, di.minimumRollConfusion(false));
	}

	// rust: minimum_roll_tentacles_escape_formula
	@Test
	public void minimumRollTentaclesEscapeFormula() {
		assertEquals(8, di.minimumRollTentaclesEscape(5, 3));
	}

	// rust: minimum_roll_shadowing_escape_formula
	@Test
	public void minimumRollShadowingEscapeFormula() {
		assertEquals(10, di.minimumRollShadowingEscape(6, 4));
	}

	// rust: is_regeneration_successful_threshold_four
	@Test
	public void isRegenerationSuccessfulThresholdFour() {
		assertTrue(di.isRegenerationSuccessful(4));
		assertFalse(di.isRegenerationSuccessful(3));
	}

	// rust: is_recovering_from_knockout_roll_four_no_babes
	@Test
	public void isRecoveringFromKnockoutRollFourNoBabes() {
		assertTrue(di.isRecoveringFromKnockout(4, 0));
		assertFalse(di.isRecoveringFromKnockout(3, 0));
	}

	// rust: is_affected_by_pitch_invasion_roll_one_fails
	@Test
	public void isAffectedByPitchInvasionRollOneFails() {
		assertFalse(di.isAffectedByPitchInvasion(1, 5));
	}

	// rust: is_double_matches_equal_dice
	@Test
	public void isDoubleMatchesEqualDice() {
		assertTrue(di.isDouble(new int[]{3, 3}));
		assertFalse(di.isDouble(new int[]{2, 3}));
		assertFalse(di.isDouble(new int[]{3}));
	}
}
