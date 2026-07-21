package com.fumbbl.ffb.client.report;

import com.fumbbl.ffb.client.TextStyle;
import com.fumbbl.ffb.report.ReportRiotousRookies;
import org.junit.jupiter.api.Test;

import java.util.List;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.mockito.BDDMockito.given;

class RiotousRookiesMessageTest extends ReportMessageTestBase {

	@Test
	public void renderHomeTeamPrintsHomeStyle() {
		given(game.getTeamHome().getId()).willReturn("home");
		given(game.getTeamHome().getName()).willReturn("home-name");

		ReportRiotousRookies report = new ReportRiotousRookies(new int[]{2, 3}, 1, "home");
		List<Run> runs = render(new RiotousRookiesMessage(), report);

		assertEquals("Riotous Rookies Roll [ 2 ][ 3 ] + 1", runs.get(0).text);
		assertEquals("home-name", runs.get(2).text);
		assertEquals(TextStyle.HOME, runs.get(2).textStyle);
		assertEquals(" hires 1 Riotous Rookies for this game", runs.get(3).text);
	}

	@Test
	public void renderAwayTeamPrintsAwayStyle() {
		given(game.getTeamHome().getId()).willReturn("home");
		given(game.getTeamAway().getName()).willReturn("away-name");

		ReportRiotousRookies report = new ReportRiotousRookies(new int[]{5, 6}, 2, "away");
		List<Run> runs = render(new RiotousRookiesMessage(), report);

		assertEquals("away-name", runs.get(2).text);
		assertEquals(TextStyle.AWAY, runs.get(2).textStyle);
		assertEquals(" hires 2 Riotous Rookies for this game", runs.get(3).text);
	}
}
