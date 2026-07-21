package com.fumbbl.ffb.client.report;

import com.fumbbl.ffb.client.TextStyle;
import com.fumbbl.ffb.report.ReportPassBlock;

import org.junit.jupiter.api.Test;

import java.util.List;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.BDDMockito.given;

class PassBlockMessageTest extends ReportMessageTestBase {

	@Test
	public void availablePrintsNothing() {
		given(game.getTeamHome().getId()).willReturn("home");

		ReportPassBlock report = new ReportPassBlock("home", true);
		List<Run> runs = render(new PassBlockMessage(), report);

		assertTrue(runs.isEmpty());
	}

	@Test
	public void unavailableHomeTeamUsesHomeStyle() {
		given(game.getTeamHome().getId()).willReturn("home");

		ReportPassBlock report = new ReportPassBlock("home", false);
		List<Run> runs = render(new PassBlockMessage(), report);

		assertEquals(TextStyle.HOME, runs.get(0).textStyle);
		assertEquals("No pass blockers in range to intercept.", runs.get(0).text);
	}

	@Test
	public void unavailableAwayTeamUsesAwayStyle() {
		given(game.getTeamHome().getId()).willReturn("home");

		ReportPassBlock report = new ReportPassBlock("away", false);
		List<Run> runs = render(new PassBlockMessage(), report);

		assertEquals(TextStyle.AWAY, runs.get(0).textStyle);
	}
}
