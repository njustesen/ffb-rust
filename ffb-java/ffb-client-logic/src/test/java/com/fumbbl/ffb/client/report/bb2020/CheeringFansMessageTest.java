package com.fumbbl.ffb.client.report.bb2020;

import com.fumbbl.ffb.client.report.ReportMessageTestBase;
import com.fumbbl.ffb.report.bb2020.ReportCheeringFans;
import org.junit.jupiter.api.Test;

import java.util.List;

import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.BDDMockito.given;

class CheeringFansMessageTest extends ReportMessageTestBase {

	private void stubTeams() {
		given(game.getTeamHome().getId()).willReturn("home");
		given(game.getTeamHome().getName()).willReturn("Home Team");
		given(game.getTeamHome().getCheerleaders()).willReturn(2);
		given(game.getTeamAway().getName()).willReturn("Away Team");
		given(game.getTeamAway().getCheerleaders()).willReturn(1);
	}

	@Test
	public void noTeamGainsPrayerWhenTeamIdEmpty() {
		stubTeams();

		ReportCheeringFans report = new ReportCheeringFans(null, false, 4, 3);
		List<Run> runs = render(new CheeringFansMessage(), report);

		assertTrue(runs.stream().anyMatch(r -> "Neither team gains a Prayer to Nuffle.".equals(r.text)));
	}

	@Test
	public void homeTeamGainsPrayerWhenAvailable() {
		stubTeams();

		ReportCheeringFans report = new ReportCheeringFans("home", true, 4, 3);
		List<Run> runs = render(new CheeringFansMessage(), report);

		assertTrue(runs.stream().anyMatch(r -> "Home Team".equals(r.text)));
		assertTrue(runs.stream().anyMatch(r -> r.text != null && r.text.contains(" gains a Prayer to Nuffle.")));
	}

	@Test
	public void awayTeamPrayerNotAvailableWhenAllInEffect() {
		stubTeams();

		ReportCheeringFans report = new ReportCheeringFans("away", false, 4, 3);
		List<Run> runs = render(new CheeringFansMessage(), report);

		assertTrue(runs.stream().anyMatch(r -> "Away Team".equals(r.text)));
		assertTrue(runs.stream().anyMatch(r -> r.text != null && r.text.contains(" would gain a Prayer to Nuffle but all are in effect.")));
	}

	@Test
	public void totalsIncludeCheerleadersFromTeam() {
		stubTeams();

		ReportCheeringFans report = new ReportCheeringFans("home", true, 4, 3);
		List<Run> runs = render(new CheeringFansMessage(), report);

		assertTrue(runs.stream().anyMatch(r -> r.text != null && r.text.contains("Rolled 4 + 2 Cheerleaders = 6.")));
		assertTrue(runs.stream().anyMatch(r -> r.text != null && r.text.contains("Rolled 3 + 1 Cheerleaders = 4.")));
	}
}
