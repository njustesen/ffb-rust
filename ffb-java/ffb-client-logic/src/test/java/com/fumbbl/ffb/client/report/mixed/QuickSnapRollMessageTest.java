package com.fumbbl.ffb.client.report.mixed;

import com.fumbbl.ffb.client.TextStyle;
import com.fumbbl.ffb.client.report.ReportMessageTestBase;
import com.fumbbl.ffb.model.Team;
import com.fumbbl.ffb.report.mixed.ReportQuickSnapRoll;
import org.junit.jupiter.api.Test;

import java.util.List;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.mockito.BDDMockito.given;

class QuickSnapRollMessageTest extends ReportMessageTestBase {

	@Test
	public void rendersRollLine() {
		Team homeTeam = game.getTeamHome();
		given(homeTeam.getId()).willReturn("home");
		given(game.getTeamById("home")).willReturn(homeTeam);
		given(homeTeam.getName()).willReturn("Home Team");

		ReportQuickSnapRoll report = new ReportQuickSnapRoll("home", 4, 2);
		List<Run> runs = render(new QuickSnapRollMessage(), report);

		assertEquals("Quick Snap Roll [ 4 ]", runs.get(0).text);
		assertEquals(TextStyle.ROLL, runs.get(0).textStyle);
	}

	@Test
	public void homeTeamUsesHomeStyle() {
		Team homeTeam = game.getTeamHome();
		given(homeTeam.getId()).willReturn("home");
		given(game.getTeamById("home")).willReturn(homeTeam);
		given(homeTeam.getName()).willReturn("Home Team");

		ReportQuickSnapRoll report = new ReportQuickSnapRoll("home", 4, 2);
		List<Run> runs = render(new QuickSnapRollMessage(), report);

		Run teamRun = runs.stream().filter(r -> "Home Team".equals(r.text)).findFirst().orElseThrow();
		assertEquals(TextStyle.HOME, teamRun.textStyle);
	}

	@Test
	public void awayTeamUsesAwayStyleAndAmountMessage() {
		Team awayTeam = game.getTeamAway();
		given(game.getTeamHome().getId()).willReturn("home");
		given(game.getTeamById("away")).willReturn(awayTeam);
		given(awayTeam.getName()).willReturn("Away Team");

		ReportQuickSnapRoll report = new ReportQuickSnapRoll("away", 4, 3);
		List<Run> runs = render(new QuickSnapRollMessage(), report);

		Run teamRun = runs.stream().filter(r -> "Away Team".equals(r.text)).findFirst().orElseThrow();
		assertEquals(TextStyle.AWAY, teamRun.textStyle);
		// The trailing println(...) emits the real text run followed by a null-text
		// terminator run, so the last captured run is the terminator, not the text.
		Run last = runs.get(runs.size() - 2);
		assertEquals(" may move 3 open players 1 square each", last.text);
	}
}
