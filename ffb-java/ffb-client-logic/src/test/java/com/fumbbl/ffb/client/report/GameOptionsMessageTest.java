package com.fumbbl.ffb.client.report;

import com.fumbbl.ffb.report.ReportGameOptions;
import org.junit.jupiter.api.Test;

import java.util.List;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;

class GameOptionsMessageTest extends ReportMessageTestBase {

	@Test
	public void renderProducesNoOutput() {
		ReportGameOptions report = new ReportGameOptions();
		List<Run> runs = render(new GameOptionsMessage(), report);
		assertTrue(runs.isEmpty());
	}

	@Test
	public void renderDoesNotChangeIndent() {
		statusReport.setIndent(3);
		ReportGameOptions report = new ReportGameOptions();
		render(new GameOptionsMessage(), report);
		assertEquals(3, statusReport.getIndent());
	}
}
