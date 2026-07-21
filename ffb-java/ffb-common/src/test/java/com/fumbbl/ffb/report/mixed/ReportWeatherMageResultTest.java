package com.fumbbl.ffb.report.mixed;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.Weather;
import com.fumbbl.ffb.report.ReportTestUtil;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

public class ReportWeatherMageResultTest {

	private ReportWeatherMageResult make() {
		return new ReportWeatherMageResult(1, Weather.NICE, ReportWeatherMageResult.Effect.CHANGED, Weather.BLIZZARD);
	}

	@Test
	public void serializationRoundTrip() {
		ReportWeatherMageResult original = make();
		JsonObject json = original.toJsonValue().asObject();
		ReportWeatherMageResult restored = new ReportWeatherMageResult().initFrom(ReportTestUtil.source(), json);
		assertEquals(original.getModifier(), restored.getModifier());
		assertEquals(original.getNewWeather(), restored.getNewWeather());
		assertEquals(original.getOldWeather(), restored.getOldWeather());
		assertEquals(original.getEffect(), restored.getEffect());
	}

	@Test
	public void toJsonValueHasReportId() {
		JsonObject json = make().toJsonValue().asObject();
		assertEquals("weatherMageResult", json.get("reportId").asString());
	}
}
