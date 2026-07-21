package com.fumbbl.ffb.client.report.mixed;

import com.fumbbl.ffb.client.report.ReportMessageTestBase;
import com.fumbbl.ffb.report.mixed.ReportEvent;
import org.junit.jupiter.api.Test;

import java.util.List;

import static org.junit.jupiter.api.Assertions.assertEquals;

class EventMessageTest extends ReportMessageTestBase {

	@Test
	public void rendersEventMessageAtIndentPlusOne() {
		ReportEvent report = new ReportEvent("Something happened.");
		List<Run> runs = render(new EventMessage(), report);

		assertEquals("Something happened.", runs.get(0).text);
	}

	@Test
	public void rendersAtBaseIndentPlusOneRegardlessOfCurrentIndent() {
		statusReport.setIndent(3);
		ReportEvent report = new ReportEvent("Kickoff!");
		List<Run> runs = render(new EventMessage(), report);

		assertEquals("Kickoff!", runs.get(0).text);
	}

	@Test
	public void rendersNullEventMessageAsIs() {
		ReportEvent report = new ReportEvent(null);
		List<Run> runs = render(new EventMessage(), report);

		assertEquals(null, runs.get(0).text);
	}
}
