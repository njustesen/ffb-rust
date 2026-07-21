package com.fumbbl.ffb.client.report.mixed;

import com.fumbbl.ffb.client.TextStyle;
import com.fumbbl.ffb.client.report.ReportMessageTestBase;
import com.fumbbl.ffb.model.Team;
import com.fumbbl.ffb.report.mixed.ReportBrilliantCoachingReRollsLost;
import org.junit.jupiter.api.Test;

import java.util.List;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.mockito.BDDMockito.given;

class BrilliantCoachingReRollsLostMessageTest extends ReportMessageTestBase {

	@Test
	public void singleRerollUsesSingularWording() {
		Team homeTeam = game.getTeamHome();
		given(game.getTeamById("home")).willReturn(homeTeam);
		given(homeTeam.getName()).willReturn("Home Team");

		ReportBrilliantCoachingReRollsLost report = new ReportBrilliantCoachingReRollsLost("home", 1);
		List<Run> runs = render(new BrilliantCoachingReRollsLostMessage(), report);

		assertEquals(" lose 1 Brilliant Coaching Re-Roll as it was not used in this drive.", runs.get(1).text);
	}

	@Test
	public void multipleRerollsUsesPluralWording() {
		Team awayTeam = game.getTeamAway();
		given(game.getTeamById("away")).willReturn(awayTeam);
		given(awayTeam.getName()).willReturn("Away Team");

		ReportBrilliantCoachingReRollsLost report = new ReportBrilliantCoachingReRollsLost("away", 3);
		List<Run> runs = render(new BrilliantCoachingReRollsLostMessage(), report);

		assertEquals(" lose 3 Brilliant Coaching Re-Rolls as they were not used in this drive.", runs.get(1).text);
	}

	@Test
	public void homeTeamUsesHomeStyle() {
		Team homeTeam = game.getTeamHome();
		given(game.getTeamById("home")).willReturn(homeTeam);
		given(homeTeam.getName()).willReturn("Home Team");

		ReportBrilliantCoachingReRollsLost report = new ReportBrilliantCoachingReRollsLost("home", 2);
		List<Run> runs = render(new BrilliantCoachingReRollsLostMessage(), report);

		assertEquals("Home Team", runs.get(0).text);
		assertEquals(TextStyle.HOME, runs.get(0).textStyle);
	}

	@Test
	public void awayTeamUsesAwayStyle() {
		Team awayTeam = game.getTeamAway();
		given(game.getTeamById("away")).willReturn(awayTeam);
		given(awayTeam.getName()).willReturn("Away Team");

		ReportBrilliantCoachingReRollsLost report = new ReportBrilliantCoachingReRollsLost("away", 2);
		List<Run> runs = render(new BrilliantCoachingReRollsLostMessage(), report);

		assertEquals("Away Team", runs.get(0).text);
		assertEquals(TextStyle.AWAY, runs.get(0).textStyle);
	}
}
