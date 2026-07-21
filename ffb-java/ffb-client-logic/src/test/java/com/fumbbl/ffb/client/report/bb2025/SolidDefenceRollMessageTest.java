package com.fumbbl.ffb.client.report.bb2025;

import com.fumbbl.ffb.client.TextStyle;
import com.fumbbl.ffb.client.report.ReportMessageTestBase;
import com.fumbbl.ffb.model.Team;
import com.fumbbl.ffb.report.ReportId;
import com.fumbbl.ffb.report.mixed.ReportSolidDefenceRoll;
import org.junit.jupiter.api.Test;
import org.mockito.Mock;

import java.util.List;
import java.util.stream.Collectors;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.mockito.BDDMockito.given;

class SolidDefenceRollMessageTest extends ReportMessageTestBase {

	@Mock
	private Team homeTeam;

	@Mock
	private Team awayTeam;

	@Test
	public void multiplePlayersMaySetupAgain() {
		given(game.getTeamHome()).willReturn(homeTeam);
		given(homeTeam.getId()).willReturn("home");
		given(homeTeam.getName()).willReturn("Team home");
		given(game.getTeamById("home")).willReturn(homeTeam);

		ReportSolidDefenceRoll report = new ReportSolidDefenceRoll("home", 5, 2);
		List<Run> runs = render(new SolidDefenceRollMessage(), report);
		List<String> texts = runs.stream().map(r -> r.text).collect(Collectors.toList());
		assertEquals(true, texts.stream().anyMatch(t -> " may select up to 2 players to setup again".equals(t)));
		Run teamRun = runs.stream().filter(r -> "Team home".equals(r.text)).findFirst().orElseThrow();
		assertEquals(TextStyle.HOME, teamRun.textStyle);
	}

	@Test
	public void singlePlayerMaySetupAgain() {
		given(game.getTeamHome()).willReturn(homeTeam);
		given(homeTeam.getId()).willReturn("home");
		given(game.getTeamAway()).willReturn(awayTeam);
		given(awayTeam.getName()).willReturn("Team away");
		given(game.getTeamById("away")).willReturn(awayTeam);

		ReportSolidDefenceRoll report = new ReportSolidDefenceRoll("away", 3, 1);
		List<Run> runs = render(new SolidDefenceRollMessage(), report);
		List<String> texts = runs.stream().map(r -> r.text).collect(Collectors.toList());
		assertEquals(true, texts.stream().anyMatch(t -> " may select 1 player to setup again".equals(t)));
		Run teamRun = runs.stream().filter(r -> "Team away".equals(r.text)).findFirst().orElseThrow();
		assertEquals(TextStyle.AWAY, teamRun.textStyle);
	}

	@Test
	public void noEligiblePlayers() {
		given(game.getTeamHome()).willReturn(homeTeam);
		given(homeTeam.getId()).willReturn("home");
		given(homeTeam.getName()).willReturn("Team home");
		given(game.getTeamById("home")).willReturn(homeTeam);

		ReportSolidDefenceRoll report = new ReportSolidDefenceRoll("home", 1, 0);
		List<Run> runs = render(new SolidDefenceRollMessage(), report);
		List<String> texts = runs.stream().map(r -> r.text).collect(Collectors.toList());
		assertEquals(true, texts.stream().anyMatch(t -> " have no eligible players, moving on to kick-off".equals(t)));
	}

	@Test
	public void reportIdIsSolidDefenceRoll() {
		assertEquals(ReportId.SOLID_DEFENCE_ROLL.getKey(), new SolidDefenceRollMessage().getKey());
	}
}
