package com.fumbbl.ffb.factory;

import com.fumbbl.ffb.Weather;

import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertNotNull;
import static org.junit.jupiter.api.Assertions.assertNull;

/**
 * Mirrors the surviving Rust tests in crates/ffb-model/src/factory/weather_factory.rs
 * for {@link WeatherFactory}.
 */
public class WeatherFactoryTest {

	@Test
	public void forNameNiceWeather() {
		assertNotNull(new WeatherFactory().forName("Nice Weather"));
	}

	@Test
	public void forShortNameNiceWeather() {
		Weather result = new WeatherFactory().forShortName("Nice");
		assertNotNull(result);
	}

	@Test
	public void forShortNameUnknown() {
		assertNull(new WeatherFactory().forShortName("XXXX"));
	}

	@Test
	public void forNameEmptyStringReturnsNone() {
		assertNull(new WeatherFactory().forName(""));
	}
}
