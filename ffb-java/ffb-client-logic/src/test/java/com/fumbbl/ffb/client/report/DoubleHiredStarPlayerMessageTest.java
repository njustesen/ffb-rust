package com.fumbbl.ffb.client.report;

import com.fumbbl.ffb.client.ParagraphStyle;
import com.fumbbl.ffb.client.TextStyle;
import com.fumbbl.ffb.report.ReportDoubleHiredStarPlayer;
import org.junit.jupiter.api.Test;

import java.util.List;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;

class DoubleHiredStarPlayerMessageTest extends ReportMessageTestBase {

	@Test
	public void rendersBoldMessageWithStarPlayerName() {
		ReportDoubleHiredStarPlayer report = new ReportDoubleHiredStarPlayer("Griff Oberwald");
		List<Run> runs = render(new DoubleHiredStarPlayerMessage(), report);

		assertEquals(2, runs.size());
		assertEquals("Star Player Griff Oberwald takes money from both teams and plays for neither.",
			runs.get(0).text);
		assertEquals(TextStyle.BOLD, runs.get(0).textStyle);
		assertEquals(ParagraphStyle.INDENT_0, runs.get(0).paragraphStyle);
	}

	@Test
	public void rendersAtCurrentIndent() {
		statusReport.setIndent(2);
		ReportDoubleHiredStarPlayer report = new ReportDoubleHiredStarPlayer("Morg 'n' Thorg");
		List<Run> runs = render(new DoubleHiredStarPlayerMessage(), report);

		assertEquals(ParagraphStyle.INDENT_2, runs.get(0).paragraphStyle);
		assertTrue(runs.get(0).text.contains("Morg 'n' Thorg"));
	}

	@Test
	public void differentStarPlayerNameReflectedInText() {
		ReportDoubleHiredStarPlayer report = new ReportDoubleHiredStarPlayer("Eldril Sidewinder");
		List<Run> runs = render(new DoubleHiredStarPlayerMessage(), report);

		assertTrue(runs.get(0).text.startsWith("Star Player Eldril Sidewinder"));
	}

	@Test
	public void reportIdIsDoubleHiredStarPlayer() {
		assertEquals("doubleHiredStarPlayer", new DoubleHiredStarPlayerMessage().getKey());
	}
}
