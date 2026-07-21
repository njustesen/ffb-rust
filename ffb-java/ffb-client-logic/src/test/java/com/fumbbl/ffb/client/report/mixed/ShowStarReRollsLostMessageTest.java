package com.fumbbl.ffb.client.report.mixed;

import com.fumbbl.ffb.client.TextStyle;
import com.fumbbl.ffb.client.report.ReportMessageTestBase;
import com.fumbbl.ffb.model.Team;
import com.fumbbl.ffb.report.mixed.ReportShowStarReRollsLost;
import org.junit.jupiter.api.Test;

import java.util.List;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.mockito.BDDMockito.given;

class ShowStarReRollsLostMessageTest extends ReportMessageTestBase {

	// java: `missing_team_still_reports_amount_line` from the Rust suite is not portable —
	// ShowStarReRollsLostMessage.render() calls `team.getName()` unconditionally (no null
	// guard); a missing team NPEs in real Java. Skipped.

	@Test
	public void singularAmountHomeTeam() {
		Team homeTeam = game.getTeamHome();
		given(game.getTeamById("home")).willReturn(homeTeam);
		given(homeTeam.getName()).willReturn("Team home");

		ReportShowStarReRollsLost report = new ReportShowStarReRollsLost("home", 1);
		List<Run> runs = render(new ShowStarReRollsLostMessage(), report);

		assertEquals("Team home", runs.get(0).text);
		assertEquals(TextStyle.HOME, runs.get(0).textStyle);
		assertEquals(" lose 1 Star of the Show Re-Roll as it was not used in this drive.", runs.get(1).text);
	}

	@Test
	public void pluralAmountAwayTeam() {
		Team awayTeam = game.getTeamAway();
		given(game.getTeamById("away")).willReturn(awayTeam);
		given(awayTeam.getName()).willReturn("Team away");

		ReportShowStarReRollsLost report = new ReportShowStarReRollsLost("away", 3);
		List<Run> runs = render(new ShowStarReRollsLostMessage(), report);

		assertEquals("Team away", runs.get(0).text);
		assertEquals(TextStyle.AWAY, runs.get(0).textStyle);
		assertEquals(" lose 3 Star of the Show Re-Rolls as they were not used in this drive.", runs.get(1).text);
	}
}
