package com.fumbbl.ffb.client.report.mixed;

import com.fumbbl.ffb.client.TextStyle;
import com.fumbbl.ffb.client.report.ReportMessageTestBase;
import com.fumbbl.ffb.report.mixed.ReportWeatherMageRoll;
import org.junit.jupiter.api.Test;

import java.util.List;

import static org.junit.jupiter.api.Assertions.assertEquals;

class WeatherMageRollMessageTest extends ReportMessageTestBase {

	@Test
	public void rendersBothDiceValues() {
		ReportWeatherMageRoll report = new ReportWeatherMageRoll(new int[] { 3, 5 });
		List<Run> runs = render(new WeatherMageRollMessage(), report);

		// run0 = weather roll text, run1 = println terminator, run2 = second line text.
		assertEquals("Weather Roll [ 3 ][ 5 ] ", runs.get(0).text);
		assertEquals(TextStyle.ROLL, runs.get(0).textStyle);
		assertEquals("The weather mage works his magic", runs.get(2).text);
	}

	@Test
	public void rendersDifferentDiceValues() {
		ReportWeatherMageRoll report = new ReportWeatherMageRoll(new int[] { 1, 6 });
		List<Run> runs = render(new WeatherMageRollMessage(), report);

		assertEquals("Weather Roll [ 1 ][ 6 ] ", runs.get(0).text);
	}

	@Test
	public void indentIsRespected() {
		statusReport.setIndent(1);
		ReportWeatherMageRoll report = new ReportWeatherMageRoll(new int[] { 2, 2 });
		List<Run> runs = render(new WeatherMageRollMessage(), report);

		assertEquals(4, runs.size());
	}
}
