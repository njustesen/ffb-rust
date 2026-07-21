package com.fumbbl.ffb.report;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.Weather;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertArrayEquals;
import static org.junit.jupiter.api.Assertions.assertEquals;

public class ReportWeatherTest {

	private ReportWeather make() {
		return new ReportWeather(Weather.NICE, new int[]{3, 4});
	}

	@Test
	public void serializationRoundTrip() {
		ReportWeather original = make();
		JsonObject json = original.toJsonValue();
		ReportWeather restored = new ReportWeather().initFrom(ReportTestUtil.source(), json);
		assertEquals(original.getWeather(), restored.getWeather());
		assertArrayEquals(original.getWeatherRoll(), restored.getWeatherRoll());
	}

	@Test
	public void toJsonValueHasReportId() {
		JsonObject json = make().toJsonValue();
		assertEquals("weather", json.get("reportId").asString());
	}
}
