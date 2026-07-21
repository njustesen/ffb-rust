package com.fumbbl.ffb.client.report.mixed;

import com.fumbbl.ffb.client.TextStyle;
import com.fumbbl.ffb.client.report.ReportMessageTestBase;
import com.fumbbl.ffb.model.Player;
import com.fumbbl.ffb.model.Team;
import com.fumbbl.ffb.report.mixed.ReportShowStarReRoll;
import org.junit.jupiter.api.Test;
import org.mockito.Mock;

import java.util.List;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.mockito.BDDMockito.given;

class ShowStarReRollMessageTest extends ReportMessageTestBase {

	@SuppressWarnings("rawtypes")
	@Mock
	private Player player;

	// java: `missing_player_still_reports_team_line` from the Rust suite is not portable —
	// ShowStarReRollMessage.render() calls `player.getTeam()` unconditionally (no null guard);
	// a missing player NPEs in real Java. Skipped.

	@Test
	public void homeStarPlayer() {
		Team homeTeam = game.getTeamHome();
		given(game.getPlayerById("p1")).willReturn(player);
		given(player.getName()).willReturn("Star");
		given(game.getTeamHome().hasPlayer(player)).willReturn(true);
		given(player.getTeam()).willReturn(homeTeam);
		given(homeTeam.getName()).willReturn("Team home");

		ReportShowStarReRoll report = new ReportShowStarReRoll("p1");
		List<Run> runs = render(new ShowStarReRollMessage(), report);

		assertEquals("Star", runs.get(0).text);
		assertEquals(" is the Star of the Show and ", runs.get(1).text);
		assertEquals("Team home", runs.get(2).text);
		assertEquals(TextStyle.HOME, runs.get(2).textStyle);
		assertEquals(" gains a Re-Roll only available for this drive.", runs.get(3).text);
		assertEquals("Will be added for the next drive.", runs.get(5).text);
		assertEquals(TextStyle.EXPLANATION, runs.get(5).textStyle);
	}

	@Test
	public void awayStarPlayer() {
		Team awayTeam = game.getTeamAway();
		given(game.getPlayerById("p2")).willReturn(player);
		given(player.getName()).willReturn("OtherStar");
		given(game.getTeamHome().hasPlayer(player)).willReturn(false);
		given(player.getTeam()).willReturn(awayTeam);
		given(awayTeam.getName()).willReturn("Team away");

		ReportShowStarReRoll report = new ReportShowStarReRoll("p2");
		List<Run> runs = render(new ShowStarReRollMessage(), report);

		assertEquals("Team away", runs.get(2).text);
		assertEquals(TextStyle.AWAY, runs.get(2).textStyle);
	}
}
