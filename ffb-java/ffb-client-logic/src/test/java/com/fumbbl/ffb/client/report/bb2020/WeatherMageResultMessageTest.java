package com.fumbbl.ffb.client.report.bb2020;

import com.fumbbl.ffb.FactoryType;
import com.fumbbl.ffb.Weather;
import com.fumbbl.ffb.client.report.ReportMessageTestBase;
import com.fumbbl.ffb.factory.MechanicsFactory;
import com.fumbbl.ffb.mechanics.GameMechanic;
import com.fumbbl.ffb.mechanics.Mechanic;
import com.fumbbl.ffb.report.mixed.ReportWeatherMageResult;
import org.junit.jupiter.api.Test;
import org.mockito.Mock;

import java.util.List;

import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.BDDMockito.given;

class WeatherMageResultMessageTest extends ReportMessageTestBase {

	@Mock
	private MechanicsFactory mechanicsFactory;

	@Mock
	private GameMechanic mechanic;

	private void stubMechanic() {
		org.mockito.Mockito.doReturn(mechanicsFactory).when(game).getFactory(FactoryType.Factory.MECHANIC);
		given(mechanicsFactory.forName(Mechanic.Type.GAME.name())).willReturn(mechanic);
		given(mechanic.weatherDescription(org.mockito.ArgumentMatchers.any())).willReturn("");
	}

	@Test
	public void noChangeReportsFailsToInfluence() {
		stubMechanic();
		ReportWeatherMageResult report = new ReportWeatherMageResult(0, null, ReportWeatherMageResult.Effect.NO_CHANGE, null);
		List<Run> runs = render(new WeatherMageResultMessage(), report);
		assertTrue(runs.stream().anyMatch(r -> r.text != null && r.text.contains("fails to influence the weather")));
	}

	@Test
	public void changedReportsNewAndOldWeather() {
		stubMechanic();
		ReportWeatherMageResult report = new ReportWeatherMageResult(
			1, Weather.BLIZZARD, ReportWeatherMageResult.Effect.CHANGED, Weather.NICE);
		List<Run> runs = render(new WeatherMageResultMessage(), report);
		assertTrue(runs.stream().anyMatch(r -> r.text != null && r.text.contains("changed to Blizzard")));
		assertTrue(runs.stream().anyMatch(r -> r.text != null && r.text.contains("return to Nice Weather")));
	}

	@Test
	public void noChoiceReportsExplanationThenChangedWeather() {
		stubMechanic();
		ReportWeatherMageResult report = new ReportWeatherMageResult(
			1, Weather.SWELTERING_HEAT, ReportWeatherMageResult.Effect.NO_CHOICE, Weather.NICE);
		List<Run> runs = render(new WeatherMageResultMessage(), report);
		assertTrue(runs.stream().anyMatch(r -> "There was only one option".equals(r.text)));
		assertTrue(runs.stream().anyMatch(r -> r.text != null && r.text.contains("changed to Sweltering Heat")));
	}

	@Test
	public void changedWeatherMentionsOpponentsTurnWhenHomePlaying() {
		stubMechanic();
		given(game.isHomePlaying()).willReturn(true);
		ReportWeatherMageResult report = new ReportWeatherMageResult(
			1, Weather.BLIZZARD, ReportWeatherMageResult.Effect.CHANGED, Weather.NICE);
		List<Run> runs = render(new WeatherMageResultMessage(), report);
		assertTrue(runs.stream().anyMatch(r -> r.text != null && r.text.contains("opponent's turn or the end of the drive")));
	}
}
