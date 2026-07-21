package com.fumbbl.ffb.client.report.bb2016;

import com.fumbbl.ffb.client.report.ReportMessageTestBase;
import com.fumbbl.ffb.report.bb2016.ReportPenaltyShootout;
import org.junit.jupiter.api.Test;

import java.util.List;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.BDDMockito.given;

class PenaltyShootoutMessageTest extends ReportMessageTestBase {

	@Test
	public void getKeyIsPenaltyShootout() {
		assertEquals("penaltyShootout", new PenaltyShootoutMessage().getKey());
	}

	@Test
	public void homeWinsWhenHigherScore() {
		given(game.getTeamHome().getName()).willReturn("Team home");

		ReportPenaltyShootout report = new ReportPenaltyShootout(5, 1, 3, 0);
		List<Run> runs = render(new PenaltyShootoutMessage(), report);

		assertTrue(runs.stream().anyMatch(r -> "Team home".equals(r.text)));
	}

	@Test
	public void awayWinsWhenHigherOrEqualScore() {
		given(game.getTeamAway().getName()).willReturn("Team away");

		ReportPenaltyShootout report = new ReportPenaltyShootout(2, 0, 4, 0);
		List<Run> runs = render(new PenaltyShootoutMessage(), report);

		assertTrue(runs.stream().anyMatch(r -> "Team away".equals(r.text)));
	}

	@Test
	public void reportsRollTotals() {
		given(game.getTeamHome().getName()).willReturn("Team home");

		ReportPenaltyShootout report = new ReportPenaltyShootout(5, 1, 3, 0);
		List<Run> runs = render(new PenaltyShootoutMessage(), report);

		assertEquals("Penalty Shootout Roll Home [5]", runs.get(0).text);
		assertEquals(" = 6", runs.get(2).text);
	}
}
