package com.fumbbl.ffb.client.report.bb2020;

import com.fumbbl.ffb.client.TextStyle;
import com.fumbbl.ffb.client.report.ReportMessageTestBase;
import com.fumbbl.ffb.model.Team;
import com.fumbbl.ffb.report.mixed.ReportSolidDefenceRoll;
import org.junit.jupiter.api.Test;
import org.mockito.Mock;

import java.util.List;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.BDDMockito.given;

class SolidDefenceRollMessageTest extends ReportMessageTestBase {

	@Mock
	private Team homeTeam;

	@Mock
	private Team awayTeam;

	@Test
	public void homeTeamRoll() {
		given(game.getTeamHome().getId()).willReturn("home");
		given(game.getTeamById("home")).willReturn(homeTeam);
		given(homeTeam.getName()).willReturn("Home Team");

		ReportSolidDefenceRoll report = new ReportSolidDefenceRoll("home", 4, 2);
		List<Run> runs = render(new SolidDefenceRollMessage(), report);
		List<String> texts = runs.stream().map(r -> r.text).collect(java.util.stream.Collectors.toList());

		assertTrue(texts.contains("Solid Defence Roll [4]"));
		assertTrue(texts.contains("Home Team"));
		assertTrue(texts.contains(" may reorganize 2 players"));
		assertTrue(texts.contains("Numbers mark original player positions."));
		Run homeRun = runs.stream().filter(r -> "Home Team".equals(r.text)).findFirst().orElseThrow();
		assertEquals(TextStyle.HOME, homeRun.textStyle);
	}

	@Test
	public void awayTeamRollUsesAwayStyle() {
		given(game.getTeamHome().getId()).willReturn("home");
		given(game.getTeamById("away")).willReturn(awayTeam);
		given(awayTeam.getName()).willReturn("Away Team");

		ReportSolidDefenceRoll report = new ReportSolidDefenceRoll("away", 6, 3);
		List<Run> runs = render(new SolidDefenceRollMessage(), report);

		Run awayRun = runs.stream().filter(r -> "Away Team".equals(r.text)).findFirst().orElseThrow();
		assertEquals(TextStyle.AWAY, awayRun.textStyle);
	}

	@Test
	public void explanationLinePresent() {
		given(game.getTeamHome().getId()).willReturn("home");
		given(game.getTeamById("home")).willReturn(homeTeam);
		given(homeTeam.getName()).willReturn("Home Team");

		ReportSolidDefenceRoll report = new ReportSolidDefenceRoll("home", 1, 0);
		List<Run> runs = render(new SolidDefenceRollMessage(), report);

		Run explanationRun = runs.stream().filter(r -> "Numbers mark original player positions.".equals(r.text)).findFirst().orElseThrow();
		assertEquals(TextStyle.EXPLANATION, explanationRun.textStyle);
	}
}
