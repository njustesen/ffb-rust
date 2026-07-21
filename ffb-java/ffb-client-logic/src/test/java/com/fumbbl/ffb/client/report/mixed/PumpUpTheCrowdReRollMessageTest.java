package com.fumbbl.ffb.client.report.mixed;

import com.fumbbl.ffb.client.TextStyle;
import com.fumbbl.ffb.client.report.ReportMessageTestBase;
import com.fumbbl.ffb.model.Player;
import com.fumbbl.ffb.model.Team;
import com.fumbbl.ffb.report.mixed.ReportPumpUpTheCrowdReRoll;
import org.junit.jupiter.api.Test;
import org.mockito.Mock;

import java.util.List;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.mockito.BDDMockito.given;

class PumpUpTheCrowdReRollMessageTest extends ReportMessageTestBase {

	@SuppressWarnings("rawtypes")
	@Mock
	private Player player;

	@Test
	public void homePlayerGainsRerollForHomeTeam() {
		Team homeTeam = game.getTeamHome();
		given(game.getPlayerById("p1")).willReturn(player);
		given(player.getName()).willReturn("Joe");
		given(game.getTeamHome().hasPlayer(player)).willReturn(true);
		given(player.getTeam()).willReturn(homeTeam);
		given(homeTeam.getName()).willReturn("Home Team");

		ReportPumpUpTheCrowdReRoll report = new ReportPumpUpTheCrowdReRoll("p1");
		List<Run> runs = render(new PumpUpTheCrowdReRollMessage(), report);

		assertEquals(true, runs.stream().anyMatch(r -> "Joe".equals(r.text)));
		Run teamRun = runs.stream().filter(r -> "Home Team".equals(r.text)).findFirst().orElseThrow();
		assertEquals(TextStyle.HOME, teamRun.textStyle);
	}

	@Test
	public void awayPlayerGainsRerollForAwayTeam() {
		Team awayTeam = game.getTeamAway();
		given(game.getPlayerById("p2")).willReturn(player);
		given(player.getName()).willReturn("Jane");
		given(game.getTeamHome().hasPlayer(player)).willReturn(false);
		given(player.getTeam()).willReturn(awayTeam);
		given(game.getTeamAway().getName()).willReturn("Away Team");

		ReportPumpUpTheCrowdReRoll report = new ReportPumpUpTheCrowdReRoll("p2");
		List<Run> runs = render(new PumpUpTheCrowdReRollMessage(), report);

		Run teamRun = runs.stream().filter(r -> "Away Team".equals(r.text)).findFirst().orElseThrow();
		assertEquals(TextStyle.AWAY, teamRun.textStyle);
	}

	@Test
	public void trailingMessageIsPrinted() {
		Team homeTeam = game.getTeamHome();
		given(game.getPlayerById("p1")).willReturn(player);
		given(player.getName()).willReturn("Joe");
		given(game.getTeamHome().hasPlayer(player)).willReturn(true);
		given(player.getTeam()).willReturn(homeTeam);
		given(homeTeam.getName()).willReturn("Home Team");

		ReportPumpUpTheCrowdReRoll report = new ReportPumpUpTheCrowdReRoll("p1");
		List<Run> runs = render(new PumpUpTheCrowdReRollMessage(), report);

		Run last = runs.get(runs.size() - 2);
		assertEquals(" gains a Re-Roll only available for this drive.", last.text);
	}
}
