package com.fumbbl.ffb.client.report.bb2016;

import com.fumbbl.ffb.client.report.ReportMessageTestBase;
import com.fumbbl.ffb.report.bb2016.ReportWinningsRoll;
import org.junit.jupiter.api.Test;

import java.util.List;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.BDDMockito.given;

class WinningsRollMessageTest extends ReportMessageTestBase {

	@Test
	public void getKeyIsWinningsRoll() {
		assertEquals("winningsRoll", new WinningsRollMessage().getKey());
	}

	@Test
	public void homeRollReportsEarnings() {
		ReportWinningsRoll report = new ReportWinningsRoll(5, 20000, 0, 0);
		List<Run> runs = render(new WinningsRollMessage(), report);

		assertTrue(runs.stream().anyMatch(r -> " earn 20,000 goldcoins.".equals(r.text)));
	}

	@Test
	public void noRollsReportsConcessionOfAway() {
		ReportWinningsRoll report = new ReportWinningsRoll(0, 15000, 0, 0);
		List<Run> runs = render(new WinningsRollMessage(), report);

		assertTrue(runs.stream().anyMatch(r -> "Winnings: Concession of Away Team".equals(r.text)));
		assertTrue(runs.stream().anyMatch(r -> " get nothing.".equals(r.text)));
	}

	@Test
	public void awayRerollReportsCoachName() {
		given(game.getTeamAway().getCoach()).willReturn("Coachaway");

		ReportWinningsRoll report = new ReportWinningsRoll(0, 0, 6, 25000);
		List<Run> runs = render(new WinningsRollMessage(), report);

		assertTrue(runs.stream().anyMatch(r -> "Coachaway".equals(r.text)));
	}
}
