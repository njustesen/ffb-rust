package com.fumbbl.ffb.client.report;

import com.fumbbl.ffb.FactoryType;
import com.fumbbl.ffb.Weather;
import com.fumbbl.ffb.client.TextStyle;
import com.fumbbl.ffb.mechanics.Mechanic;
import com.fumbbl.ffb.mechanics.bb2025.GameMechanic;
import com.fumbbl.ffb.report.ReportWeather;
import org.junit.jupiter.api.Test;

import java.util.List;
import java.util.stream.Collectors;

import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.BDDMockito.given;

class WeatherMessageTest extends ReportMessageTestBase {

	private void stubMechanic() {
		given(game.getFactory(FactoryType.Factory.MECHANIC).forName(Mechanic.Type.GAME.name()))
			.willReturn(new GameMechanic());
	}

	@Test
	public void niceWeatherRendersRollAndDescription() {
		stubMechanic();

		ReportWeather report = new ReportWeather(Weather.NICE, new int[] {3, 4});
		List<Run> runs = render(new WeatherMessage(), report);

		List<String> texts = runs.stream().map(r -> r.text).collect(Collectors.toList());
		assertTrue(texts.contains("Weather Roll [ 3 ][ 4 ] "));
		assertTrue(texts.contains("Weather is Nice Weather"));
	}

	@Test
	public void explanationStyleIsUsedForWeatherDescription() {
		stubMechanic();

		ReportWeather report = new ReportWeather(Weather.BLIZZARD, new int[] {1, 6});
		List<Run> runs = render(new WeatherMessage(), report);

		boolean hasExplanation = runs.stream().anyMatch(r -> r.textStyle == TextStyle.EXPLANATION);
		assertTrue(hasExplanation);
	}

	@Test
	public void swelteringHeatRendersCorrectName() {
		stubMechanic();

		ReportWeather report = new ReportWeather(Weather.SWELTERING_HEAT, new int[] {2, 2});
		List<Run> runs = render(new WeatherMessage(), report);

		List<String> texts = runs.stream().map(r -> r.text).collect(Collectors.toList());
		assertTrue(texts.contains("Weather is Sweltering Heat"));
	}
}
