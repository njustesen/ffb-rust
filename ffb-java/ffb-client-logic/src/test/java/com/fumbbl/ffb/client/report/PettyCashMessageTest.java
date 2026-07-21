package com.fumbbl.ffb.client.report;

import com.fumbbl.ffb.model.Team;
import com.fumbbl.ffb.report.ReportPettyCash;

import org.junit.jupiter.api.Test;
import org.mockito.Mock;

import java.util.List;

import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.BDDMockito.given;

class PettyCashMessageTest extends ReportMessageTestBase {

	@Mock
	private Team homeTeam;

	@Mock
	private Team awayTeam;

	private void wireTeams(int homeTreasury, int awayTreasury) {
		given(game.getTeamHome()).willReturn(homeTeam);
		given(game.getTeamAway()).willReturn(awayTeam);
		given(homeTeam.getId()).willReturn("home");
		given(homeTeam.getName()).willReturn("home");
		given(homeTeam.getTreasury()).willReturn(homeTreasury);
		given(awayTeam.getId()).willReturn("away");
		given(awayTeam.getName()).willReturn("away");
		given(awayTeam.getTreasury()).willReturn(awayTreasury);
	}

	@Test
	public void firstReportPrintsHeaderOnce() {
		wireTeams(1000, 1000);

		ReportPettyCash report = new ReportPettyCash("home", 500);
		List<Run> runs = render(new PettyCashMessage(), report);

		assertTrue(statusReport.fPettyCashReportReceived);
		assertTrue(runs.stream().anyMatch(r -> "Transfer Petty Cash".equals(r.text)));
	}

	@Test
	public void secondReportDoesNotRepeatHeader() {
		wireTeams(1000, 1000);
		statusReport.fPettyCashReportReceived = true;

		ReportPettyCash report = new ReportPettyCash("home", 500);
		List<Run> runs = render(new PettyCashMessage(), report);

		assertTrue(runs.stream().noneMatch(r -> "Transfer Petty Cash".equals(r.text)));
	}

	@Test
	public void zeroGoldSaysNothing() {
		wireTeams(1000, 1000);

		ReportPettyCash report = new ReportPettyCash("away", 0);
		List<Run> runs = render(new PettyCashMessage(), report);

		assertTrue(runs.stream().anyMatch(r -> " transfers nothing from the Treasury into Petty Cash.".equals(r.text)));
	}

	@Test
	public void goldOverTreasuryReportsUnderdogBonus() {
		wireTeams(100, 1000);

		ReportPettyCash report = new ReportPettyCash("home", 500);
		List<Run> runs = render(new PettyCashMessage(), report);

		assertTrue(runs.stream().anyMatch(r -> "They received an extra 400 gold for being the underdog.".equals(r.text)));
	}

	@Test
	public void goldFormatsWithThousandsSeparator() {
		wireTeams(1_000_000, 1000);

		ReportPettyCash report = new ReportPettyCash("home", 50_000);
		List<Run> runs = render(new PettyCashMessage(), report);

		assertTrue(runs.stream().anyMatch(r -> " transfers 50,000 gold from the Treasury into Petty Cash.".equals(r.text)));
	}
}
