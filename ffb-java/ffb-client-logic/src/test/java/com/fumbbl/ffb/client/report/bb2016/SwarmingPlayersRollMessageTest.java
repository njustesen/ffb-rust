package com.fumbbl.ffb.client.report.bb2016;

import com.fumbbl.ffb.client.TextStyle;
import com.fumbbl.ffb.client.report.ReportMessageTestBase;
import com.fumbbl.ffb.report.mixed.ReportSwarmingRoll;
import org.junit.jupiter.api.Test;

import java.util.List;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.BDDMockito.given;

class SwarmingPlayersRollMessageTest extends ReportMessageTestBase {

	@Test
	public void getKeyIsSwarmingPlayersRoll() {
		assertEquals("swarmingPlayersRoll", new SwarmingPlayersRollMessage().getKey());
	}

	@Test
	public void homeTeamReportsHomeBoldStyle() {
		com.fumbbl.ffb.model.Team home = game.getTeamHome();
		given(game.getTeamById("home")).willReturn(home);
		given(home.getName()).willReturn("Team home");

		ReportSwarmingRoll report = new ReportSwarmingRoll("home", 2, 4, 3);
		List<Run> runs = render(new SwarmingPlayersRollMessage(), report);

		assertEquals("Swarming Roll [2]", runs.get(0).text);
		assertTrue(runs.stream().anyMatch(r -> r.textStyle == TextStyle.HOME_BOLD));
	}

	@Test
	public void awayTeamReportsAwayBoldStyle() {
		com.fumbbl.ffb.model.Team away = game.getTeamAway();
		given(game.getTeamById("away")).willReturn(away);
		given(away.getName()).willReturn("Team away");

		ReportSwarmingRoll report = new ReportSwarmingRoll("away", 1, 2, 1);
		List<Run> runs = render(new SwarmingPlayersRollMessage(), report);

		assertTrue(runs.stream().anyMatch(r -> r.textStyle == TextStyle.AWAY_BOLD));
	}
}
