package com.fumbbl.ffb.client.report.bb2020;

import com.fumbbl.ffb.client.TextStyle;
import com.fumbbl.ffb.client.report.ReportMessageTestBase;
import com.fumbbl.ffb.model.Team;
import com.fumbbl.ffb.report.mixed.ReportBlitzRoll;
import org.junit.jupiter.api.Test;

import java.util.List;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.BDDMockito.given;

class BlitzRollMessageTest extends ReportMessageTestBase {

	@Test
	public void homeTeamBlitzRollPrintsRollAndAmount() {
		Team home = game.getTeamHome();
		given(home.getId()).willReturn("home");
		given(game.getTeamById("home")).willReturn(home);
		given(home.getName()).willReturn("Home Team");

		ReportBlitzRoll report = new ReportBlitzRoll("home", 4, 2);
		List<Run> runs = render(new BlitzRollMessage(), report);

		assertTrue(runs.stream().anyMatch(r -> r.text != null && r.text.contains("Blitz Roll [ 4 ]")));
		assertTrue(runs.stream().anyMatch(r -> "Home Team".equals(r.text)));
		assertTrue(runs.stream().anyMatch(r -> r.text != null && r.text.contains("may activate 2 open players")));
	}

	@Test
	public void awayTeamUsesAwayTextStyle() {
		Team away = game.getTeamAway();
		given(game.getTeamHome().getId()).willReturn("home");
		given(game.getTeamById("away")).willReturn(away);
		given(away.getName()).willReturn("Away Team");

		ReportBlitzRoll report = new ReportBlitzRoll("away", 6, 3);
		List<Run> runs = render(new BlitzRollMessage(), report);

		Run teamRun = runs.stream().filter(r -> "Away Team".equals(r.text)).findFirst().orElseThrow();
		assertEquals(TextStyle.AWAY, teamRun.textStyle);
	}

	@Test
	public void homeTeamUsesHomeTextStyle() {
		Team home = game.getTeamHome();
		given(home.getId()).willReturn("home");
		given(game.getTeamById("home")).willReturn(home);
		given(home.getName()).willReturn("Home Team");

		ReportBlitzRoll report = new ReportBlitzRoll("home", 5, 1);
		List<Run> runs = render(new BlitzRollMessage(), report);

		Run teamRun = runs.stream().filter(r -> "Home Team".equals(r.text)).findFirst().orElseThrow();
		assertEquals(TextStyle.HOME, teamRun.textStyle);
	}
}
