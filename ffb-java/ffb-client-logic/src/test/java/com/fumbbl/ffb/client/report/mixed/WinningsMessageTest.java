package com.fumbbl.ffb.client.report.mixed;

import com.fumbbl.ffb.client.TextStyle;
import com.fumbbl.ffb.client.report.ReportMessageTestBase;
import com.fumbbl.ffb.report.mixed.ReportWinnings;
import org.junit.jupiter.api.Test;

import java.util.List;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.mockito.BDDMockito.given;

class WinningsMessageTest extends ReportMessageTestBase {

	@Test
	public void rendersHomeAndAwayWinnings() {
		given(game.getTeamHome().getName()).willReturn("Team home");
		given(game.getTeamAway().getName()).willReturn("Team away");

		ReportWinnings report = new ReportWinnings(50_000, 30_000);
		List<Run> runs = render(new WinningsMessage(), report);

		// run0 = home name, run1 = home earnings text, run2 = terminator,
		// run3 = away name, run4 = away earnings text, run5 = terminator.
		assertEquals("Team home", runs.get(0).text);
		assertEquals(TextStyle.HOME_BOLD, runs.get(0).textStyle);
		assertEquals(" earns 50,000 gold.", runs.get(1).text);
		assertEquals("Team away", runs.get(3).text);
		assertEquals(TextStyle.AWAY_BOLD, runs.get(3).textStyle);
		assertEquals(" earns 30,000 gold.", runs.get(4).text);
	}

	@Test
	public void smallAmountsHaveNoSeparator() {
		given(game.getTeamHome().getName()).willReturn("Team home");
		given(game.getTeamAway().getName()).willReturn("Team away");

		ReportWinnings report = new ReportWinnings(42, 0);
		List<Run> runs = render(new WinningsMessage(), report);

		assertEquals(" earns 42 gold.", runs.get(1).text);
		assertEquals(" earns 0 gold.", runs.get(4).text);
	}

	@Test
	public void largeAmountHasMultipleSeparators() {
		given(game.getTeamHome().getName()).willReturn("Team home");
		given(game.getTeamAway().getName()).willReturn("Team away");

		ReportWinnings report = new ReportWinnings(2_130_000, 999_000);
		List<Run> runs = render(new WinningsMessage(), report);

		assertEquals(" earns 2,130,000 gold.", runs.get(1).text);
		assertEquals(" earns 999,000 gold.", runs.get(4).text);
	}
}
