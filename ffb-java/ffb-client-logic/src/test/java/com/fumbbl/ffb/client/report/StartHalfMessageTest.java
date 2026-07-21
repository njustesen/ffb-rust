package com.fumbbl.ffb.client.report;

import com.fumbbl.ffb.client.ParagraphStyle;
import com.fumbbl.ffb.client.TextStyle;
import com.fumbbl.ffb.report.ReportStartHalf;
import org.junit.jupiter.api.Test;

import java.util.List;

import static org.junit.jupiter.api.Assertions.assertEquals;

class StartHalfMessageTest extends ReportMessageTestBase {

	@Test
	public void firstHalf() {
		ReportStartHalf report = new ReportStartHalf(1);
		List<Run> runs = render(new StartHalfMessage(), report);

		assertEquals("Starting 1st half", runs.get(0).text);
		assertEquals(ParagraphStyle.SPACE_ABOVE_BELOW, runs.get(0).paragraphStyle);
		assertEquals(TextStyle.TURN, runs.get(0).textStyle);
	}

	@Test
	public void secondHalf() {
		ReportStartHalf report = new ReportStartHalf(2);
		List<Run> runs = render(new StartHalfMessage(), report);

		assertEquals("Starting 2nd half", runs.get(0).text);
	}

	@Test
	public void overtime() {
		ReportStartHalf report = new ReportStartHalf(3);
		List<Run> runs = render(new StartHalfMessage(), report);

		assertEquals("Starting Overtime", runs.get(0).text);
	}

	@Test
	public void reportIdIsStartHalf() {
		assertEquals("startHalf", new StartHalfMessage().getKey());
	}
}
