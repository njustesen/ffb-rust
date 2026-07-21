package com.fumbbl.ffb.report.mixed;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.report.ReportTestUtil;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertArrayEquals;
import static org.junit.jupiter.api.Assertions.assertEquals;

public class ReportWeatherMageRollTest {

	private ReportWeatherMageRoll make() {
		return new ReportWeatherMageRoll(new int[]{3, 4});
	}

	@Test
	public void serializationRoundTrip() {
		ReportWeatherMageRoll original = make();
		JsonObject json = original.toJsonValue().asObject();
		ReportWeatherMageRoll restored = new ReportWeatherMageRoll().initFrom(ReportTestUtil.source(), json);
		assertArrayEquals(original.getWeatherRoll(), restored.getWeatherRoll());
	}

	@Test
	public void toJsonValueHasReportId() {
		JsonObject json = make().toJsonValue().asObject();
		assertEquals("weatherMageRoll", json.get("reportId").asString());
	}
}
