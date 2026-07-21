package com.fumbbl.ffb.client.report;

import com.fumbbl.ffb.client.ParagraphStyle;
import com.fumbbl.ffb.client.TextStyle;
import com.fumbbl.ffb.report.ReportBombOutOfBounds;
import org.junit.jupiter.api.Test;

import java.util.List;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNull;

class BombOutOfBoundsMessageTest extends ReportMessageTestBase {

	@Test
	public void rendersBoldMessage() {
		ReportBombOutOfBounds report = new ReportBombOutOfBounds();
		List<Run> runs = render(new BombOutOfBoundsMessage(), report);

		assertEquals("Bomb scattered out of bounds.", runs.get(0).text);
		assertEquals(TextStyle.BOLD, runs.get(0).textStyle);
	}

	@Test
	public void emitsTerminatorRun() {
		ReportBombOutOfBounds report = new ReportBombOutOfBounds();
		List<Run> runs = render(new BombOutOfBoundsMessage(), report);

		assertEquals(2, runs.size());
		assertNull(runs.get(1).text);
	}

	@Test
	public void honorsCurrentIndent() {
		ReportBombOutOfBounds report = new ReportBombOutOfBounds();
		statusReport.setIndent(3);
		List<Run> runs = render(new BombOutOfBoundsMessage(), report);

		assertEquals(ParagraphStyle.INDENT_3, runs.get(0).paragraphStyle);
	}
}
