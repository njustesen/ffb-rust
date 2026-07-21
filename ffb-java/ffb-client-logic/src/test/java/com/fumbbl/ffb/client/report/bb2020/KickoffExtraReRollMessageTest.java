package com.fumbbl.ffb.client.report.bb2020;

import com.fumbbl.ffb.client.report.ReportMessageTestBase;
import com.fumbbl.ffb.report.mixed.ReportKickoffExtraReRoll;
import org.junit.jupiter.api.Test;

import java.util.List;

import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.BDDMockito.given;

class KickoffExtraReRollMessageTest extends ReportMessageTestBase {

	@Test
	public void noTeamGainsReroll() {
		ReportKickoffExtraReRoll report = new ReportKickoffExtraReRoll(3, 2, null);
		List<Run> runs = render(new KickoffExtraReRollMessage(), report);
		assertTrue(runs.stream().anyMatch(r -> "Neither team gains a Re-Roll.".equals(r.text)));
	}

	@Test
	public void homeTeamGainsReroll() {
		given(game.getTeamHome().getId()).willReturn("home");
		ReportKickoffExtraReRoll report = new ReportKickoffExtraReRoll(3, 2, "home");
		List<Run> runs = render(new KickoffExtraReRollMessage(), report);
		assertTrue(runs.stream().anyMatch(r -> "Team ".equals(r.text)));
		assertTrue(runs.stream().anyMatch(r -> " gains a Re-Roll only available for this drive.".equals(r.text)));
	}

	@Test
	public void coachBannedUsesMinusOne() {
		given(game.getTurnDataHome().isCoachBanned()).willReturn(true);
		ReportKickoffExtraReRoll report = new ReportKickoffExtraReRoll(3, 2, null);
		List<Run> runs = render(new KickoffExtraReRollMessage(), report);
		assertTrue(runs.stream().anyMatch(r -> r.text != null && r.text.contains("- 1 Banned Coach")));
	}
}
