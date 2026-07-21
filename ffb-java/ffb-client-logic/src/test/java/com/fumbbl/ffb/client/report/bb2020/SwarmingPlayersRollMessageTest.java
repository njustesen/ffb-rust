package com.fumbbl.ffb.client.report.bb2020;

import com.fumbbl.ffb.client.TextStyle;
import com.fumbbl.ffb.client.report.ReportMessageTestBase;
import com.fumbbl.ffb.model.Team;
import com.fumbbl.ffb.report.mixed.ReportSwarmingRoll;
import org.junit.jupiter.api.Test;

import java.util.List;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.BDDMockito.given;

class SwarmingPlayersRollMessageTest extends ReportMessageTestBase {

	@Test
	public void legacyPathUsedWhenLimitNegative() {
		Team homeTeam = game.getTeamHome();
		given(game.getTeamById("home")).willReturn(homeTeam);
		given(homeTeam.getName()).willReturn("Home Team");

		ReportSwarmingRoll report = new ReportSwarmingRoll("home", 3, 5, -1);
		List<Run> runs = render(new SwarmingPlayersRollMessage(), report);
		List<String> texts = runs.stream().map(r -> r.text).collect(java.util.stream.Collectors.toList());

		assertTrue(texts.contains("Swarming Roll [3]"));
		assertTrue(texts.contains(" are allowed to place 3 swarming players."));
	}

	@Test
	public void newPathZeroAmount() {
		Team awayTeam = game.getTeamAway();
		given(game.getTeamById("away")).willReturn(awayTeam);
		given(awayTeam.getName()).willReturn("Away Team");

		ReportSwarmingRoll report = new ReportSwarmingRoll("away", 0, 6, 4);
		List<Run> runs = render(new SwarmingPlayersRollMessage(), report);
		List<String> texts = runs.stream().map(r -> r.text).collect(java.util.stream.Collectors.toList());

		assertTrue(texts.contains("Swarming Roll [6]"));
		assertTrue(texts.contains(" have 4 swarming players on the pitch."));
		assertTrue(texts.contains("They are not allowed to place any swarming players."));
		Run awayRun = runs.stream().filter(r -> "Away Team".equals(r.text)).findFirst().orElseThrow();
		assertEquals(TextStyle.AWAY_BOLD, awayRun.textStyle);
	}

	@Test
	public void newPathPositiveAmount() {
		Team homeTeam = game.getTeamHome();
		given(game.getTeamById("home")).willReturn(homeTeam);
		given(homeTeam.getName()).willReturn("Home Team");

		ReportSwarmingRoll report = new ReportSwarmingRoll("home", 2, 6, 4);
		List<Run> runs = render(new SwarmingPlayersRollMessage(), report);
		List<String> texts = runs.stream().map(r -> r.text).collect(java.util.stream.Collectors.toList());

		assertTrue(texts.contains("They are allowed to place 2 swarming players."));
		Run homeRun = runs.stream().filter(r -> "Home Team".equals(r.text)).findFirst().orElseThrow();
		assertEquals(TextStyle.HOME_BOLD, homeRun.textStyle);
	}
}
