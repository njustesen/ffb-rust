package com.fumbbl.ffb.client.report.bb2025;

import com.fumbbl.ffb.client.TextStyle;
import com.fumbbl.ffb.client.report.ReportMessageTestBase;
import com.fumbbl.ffb.model.Team;
import com.fumbbl.ffb.report.ReportId;
import com.fumbbl.ffb.report.bb2025.ReportSwarmingRoll;
import org.junit.jupiter.api.Test;
import org.mockito.Mock;

import java.util.List;
import java.util.stream.Collectors;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.mockito.BDDMockito.given;

class SwarmingPlayersRollMessageTest extends ReportMessageTestBase {

	@Mock
	private Team homeTeam;

	@Mock
	private Team awayTeam;

	@Test
	public void homeTeamUsesHomeBoldStyle() {
		given(game.getTeamHome()).willReturn(homeTeam);
		given(homeTeam.getName()).willReturn("Team home");
		given(game.getTeamById("home")).willReturn(homeTeam);

		ReportSwarmingRoll report = new ReportSwarmingRoll("home", 3);
		List<Run> runs = render(new SwarmingPlayersRollMessage(), report);
		Run teamRun = runs.stream().filter(r -> "Team home".equals(r.text)).findFirst().orElseThrow();
		assertEquals(TextStyle.HOME_BOLD, teamRun.textStyle);
	}

	@Test
	public void awayTeamUsesAwayBoldStyle() {
		given(game.getTeamHome()).willReturn(homeTeam);
		given(game.getTeamById("away")).willReturn(awayTeam);
		given(awayTeam.getName()).willReturn("Team away");

		ReportSwarmingRoll report = new ReportSwarmingRoll("away", 5);
		List<Run> runs = render(new SwarmingPlayersRollMessage(), report);
		Run teamRun = runs.stream().filter(r -> "Team away".equals(r.text)).findFirst().orElseThrow();
		assertEquals(TextStyle.AWAY_BOLD, teamRun.textStyle);
		List<String> texts = runs.stream().map(r -> r.text).collect(Collectors.toList());
		assertEquals(true, texts.stream().anyMatch(t -> " are allowed to place 5 swarming players.".equals(t)));
	}

	@Test
	public void rollReportedInHeader() {
		given(game.getTeamHome()).willReturn(homeTeam);
		given(game.getTeamById("home")).willReturn(homeTeam);
		given(homeTeam.getName()).willReturn("Team home");

		ReportSwarmingRoll report = new ReportSwarmingRoll("home", 2);
		List<Run> runs = render(new SwarmingPlayersRollMessage(), report);
		List<String> texts = runs.stream().map(r -> r.text).collect(Collectors.toList());
		assertEquals(true, texts.stream().anyMatch(t -> "Swarming Roll [2]".equals(t)));
	}

	@Test
	public void reportIdIsSwarmingPlayersRoll() {
		assertEquals(ReportId.SWARMING_PLAYERS_ROLL.getKey(), new SwarmingPlayersRollMessage().getKey());
	}
}
