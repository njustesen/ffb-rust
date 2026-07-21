package com.fumbbl.ffb.client.report.bb2025;

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

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.ArgumentMatchers.any;
import static org.mockito.BDDMockito.given;

class WeatherMageResultMessageTest extends ReportMessageTestBase {

	@Mock
	private MechanicsFactory mechanicsFactory;

	@Mock
	private GameMechanic mechanic;

	private void stubMechanic() {
		// game is a RETURNS_DEEP_STUBS mock; getFactory(...) has a bound generic return type
		// (T extends INamedObjectFactory<?>), so given(game.<MechanicsFactory>getFactory(...))
		// forces a checkcast to MechanicsFactory against the deep-stub's default proxy (which
		// only implements INamedObjectFactory) BEFORE the stubbing is even recorded, causing a
		// ClassCastException. doReturn(...).when(...) avoids evaluating that generic checkcast.
		org.mockito.Mockito.doReturn(mechanicsFactory).when(game).getFactory(FactoryType.Factory.MECHANIC);
		given(mechanicsFactory.forName(Mechanic.Type.GAME.name())).willReturn(mechanic);
		given(mechanic.weatherDescription(any())).willReturn("");
	}

	@Test
	public void getKeyIsWeatherMageResult() {
		assertEquals("weatherMageResult", new WeatherMageResultMessage().getKey());
	}

	@Test
	public void noChangeReportsFailsToInfluence() {
		// the handler resolves `mechanic` unconditionally before the effect switch, so it must
		// be stubbed even though the NO_CHANGE branch never reads the variable - otherwise the
		// unstubbed getFactory(...).forName(...) chain produces a value that cannot be cast to
		// GameMechanic and the render throws a ClassCastException.
		stubMechanic();
		ReportWeatherMageResult report = new ReportWeatherMageResult(0, null, ReportWeatherMageResult.Effect.NO_CHANGE, null);
		List<Run> runs = render(new WeatherMageResultMessage(), report);

		assertTrue(runs.stream().anyMatch(r -> r.text != null && r.text.contains("fails to influence the weather")));
	}

	@Test
	public void changedReportsNewWeatherAndDescription() {
		stubMechanic();
		ReportWeatherMageResult report = new ReportWeatherMageResult(1, Weather.NICE, ReportWeatherMageResult.Effect.CHANGED, Weather.SWELTERING_HEAT);
		List<Run> runs = render(new WeatherMageResultMessage(), report);

		assertTrue(runs.stream().anyMatch(r -> "The weather is changed to Nice Weather".equals(r.text)));
	}

	@Test
	public void noChoiceReportsOnlyOneOptionThenChangedWeather() {
		stubMechanic();
		ReportWeatherMageResult report = new ReportWeatherMageResult(1, Weather.NICE, ReportWeatherMageResult.Effect.NO_CHOICE, null);
		List<Run> runs = render(new WeatherMageResultMessage(), report);

		assertTrue(runs.stream().anyMatch(r -> "There was only one option".equals(r.text)));
		assertTrue(runs.stream().anyMatch(r -> "The weather is changed to Nice Weather".equals(r.text)));
	}
}
