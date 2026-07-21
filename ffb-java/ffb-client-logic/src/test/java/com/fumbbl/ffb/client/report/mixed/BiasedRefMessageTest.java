package com.fumbbl.ffb.client.report.mixed;

import com.fumbbl.ffb.client.report.ReportMessageTestBase;
import com.fumbbl.ffb.report.mixed.ReportBiasedRef;
import org.junit.jupiter.api.Test;

import java.util.List;

import static org.junit.jupiter.api.Assertions.assertEquals;

class BiasedRefMessageTest extends ReportMessageTestBase {

	@Test
	public void foulSpottedReportsSpotted() {
		ReportBiasedRef report = new ReportBiasedRef(4, true);
		List<Run> runs = render(new BiasedRefMessage(), report);

		assertEquals("Biased Roll [ 4 ]", runs.get(0).text);
		assertEquals("The biased referee spots the foul.", runs.get(2).text);
	}

	@Test
	public void foulNotSpottedReportsNotSpotted() {
		ReportBiasedRef report = new ReportBiasedRef(2, false);
		List<Run> runs = render(new BiasedRefMessage(), report);

		assertEquals("The biased referee does not spot the foul.", runs.get(2).text);
	}
}
