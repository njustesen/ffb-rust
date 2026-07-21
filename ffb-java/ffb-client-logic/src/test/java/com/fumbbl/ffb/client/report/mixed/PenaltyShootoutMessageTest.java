package com.fumbbl.ffb.client.report.mixed;

import com.fumbbl.ffb.client.TextStyle;
import com.fumbbl.ffb.client.report.ReportMessageTestBase;
import com.fumbbl.ffb.model.Team;
import com.fumbbl.ffb.report.mixed.ReportPenaltyShootout;
import org.junit.jupiter.api.Test;
import org.mockito.Mock;

import java.util.List;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.BDDMockito.given;

class PenaltyShootoutMessageTest extends ReportMessageTestBase {

	@Mock
	private Team homeTeam;

	@Mock
	private Team awayTeam;

	@Test
	public void homeWinsPenaltyWithWinningTeam() {
		given(game.getTeamHome()).willReturn(homeTeam);
		given(game.getTeamAway()).willReturn(awayTeam);
		given(homeTeam.getName()).willReturn("Team home");
		given(awayTeam.getName()).willReturn("Team away");
		given(game.getTeamById("home")).willReturn(homeTeam);

		ReportPenaltyShootout report = new ReportPenaltyShootout(4, 1, 3, 0, true, "2", "home");
		List<Run> runs = render(new PenaltyShootoutMessage(), report);

		assertEquals("2 Penalty Shootout Rolls: Home [4] Away [3]", runs.get(0).text);
		assertEquals("Team home", runs.get(2).text);
		assertEquals(TextStyle.HOME, runs.get(2).textStyle);
		boolean hasSuddenDeath = runs.stream().anyMatch(r -> " win sudden death".equals(r.text));
		assertTrue(hasSuddenDeath);
	}

	@Test
	public void awayWinsPenaltyNoWinningTeam() {
		given(game.getTeamHome()).willReturn(homeTeam);
		given(game.getTeamAway()).willReturn(awayTeam);
		given(homeTeam.getName()).willReturn("Team home");
		given(awayTeam.getName()).willReturn("Team away");

		ReportPenaltyShootout report = new ReportPenaltyShootout(2, 1, 5, 2, false, "1", null);
		List<Run> runs = render(new PenaltyShootoutMessage(), report);

		assertEquals("Team away", runs.get(2).text);
		assertEquals(TextStyle.AWAY, runs.get(2).textStyle);
		boolean hasSuddenDeath = runs.stream().anyMatch(r -> " win sudden death".equals(r.text));
		assertFalse(hasSuddenDeath);
	}

	@Test
	public void noWinnerPenaltyRerolled() {
		given(game.getTeamHome()).willReturn(homeTeam);
		given(game.getTeamAway()).willReturn(awayTeam);

		ReportPenaltyShootout report = new ReportPenaltyShootout(3, 3, 3, 3, null, "3", null);
		List<Run> runs = render(new PenaltyShootoutMessage(), report);

		assertEquals("Penalty is rerolled", runs.get(2).text);
	}

	@Test
	public void emptyWinningTeamStringIsNotProvided() {
		given(game.getTeamHome()).willReturn(homeTeam);
		given(game.getTeamAway()).willReturn(awayTeam);
		given(homeTeam.getName()).willReturn("Team home");
		given(awayTeam.getName()).willReturn("Team away");

		ReportPenaltyShootout report = new ReportPenaltyShootout(1, 0, 1, 0, true, "1", "");
		List<Run> runs = render(new PenaltyShootoutMessage(), report);

		boolean hasSuddenDeath = runs.stream().anyMatch(r -> " win sudden death".equals(r.text));
		assertFalse(hasSuddenDeath);
	}
}
