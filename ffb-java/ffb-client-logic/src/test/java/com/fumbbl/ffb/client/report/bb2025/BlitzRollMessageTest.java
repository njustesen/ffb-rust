package com.fumbbl.ffb.client.report.bb2025;

import com.fumbbl.ffb.client.TextStyle;
import com.fumbbl.ffb.client.report.ReportMessageTestBase;
import com.fumbbl.ffb.model.Team;
import com.fumbbl.ffb.report.ReportId;
import com.fumbbl.ffb.report.mixed.ReportBlitzRoll;
import org.junit.jupiter.api.Test;
import org.mockito.Mock;

import java.util.List;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.mockito.BDDMockito.given;

class BlitzRollMessageTest extends ReportMessageTestBase {

	@Mock
	private Team homeTeam;

	@Mock
	private Team awayTeam;

	@Test
	public void homeTeamUsesHomeStyle() {
		given(game.getTeamHome()).willReturn(homeTeam);
		given(homeTeam.getId()).willReturn("home");
		given(homeTeam.getName()).willReturn("Team home");
		given(game.getTeamById("home")).willReturn(homeTeam);

		ReportBlitzRoll report = new ReportBlitzRoll("home", 5, 2);
		List<Run> runs = render(new BlitzRollMessage(), report);
		Run teamRun = runs.stream().filter(r -> "Team home".equals(r.text)).findFirst().orElseThrow();
		assertEquals(TextStyle.HOME, teamRun.textStyle);
	}

	@Test
	public void awayTeamUsesAwayStyle() {
		given(game.getTeamHome()).willReturn(homeTeam);
		given(homeTeam.getId()).willReturn("home");
		given(game.getTeamAway()).willReturn(awayTeam);
		given(awayTeam.getName()).willReturn("Team away");
		given(game.getTeamById("away")).willReturn(awayTeam);

		ReportBlitzRoll report = new ReportBlitzRoll("away", 4, 3);
		List<Run> runs = render(new BlitzRollMessage(), report);
		Run teamRun = runs.stream().filter(r -> "Team away".equals(r.text)).findFirst().orElseThrow();
		assertEquals(TextStyle.AWAY, teamRun.textStyle);
	}

	@Test
	public void rollAndAmountText() {
		given(game.getTeamHome()).willReturn(homeTeam);
		given(homeTeam.getId()).willReturn("home");
		given(homeTeam.getName()).willReturn("Team home");
		given(game.getTeamById("home")).willReturn(homeTeam);

		ReportBlitzRoll report = new ReportBlitzRoll("home", 6, 4);
		List<Run> runs = render(new BlitzRollMessage(), report);
		List<String> texts = runs.stream().map(r -> r.text).collect(java.util.stream.Collectors.toList());
		assertEquals(true, texts.stream().anyMatch(t -> "Charge! Roll [ 6 ]".equals(t)));
		assertEquals(true, texts.stream().anyMatch(t -> t != null && t.contains("select 4 open players")));
	}

	@Test
	public void reportIdIsBlitzRoll() {
		assertEquals(ReportId.BLITZ_ROLL.getKey(), new BlitzRollMessage().getKey());
	}
}
