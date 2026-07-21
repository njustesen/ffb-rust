package com.fumbbl.ffb.client.report.bb2025;

import com.fumbbl.ffb.client.report.ReportMessageTestBase;
import com.fumbbl.ffb.inducement.Usage;
import com.fumbbl.ffb.report.ReportId;
import com.fumbbl.ffb.report.bb2025.ReportCheeringFans;
import org.junit.jupiter.api.Test;

import java.util.Collections;
import java.util.HashSet;
import java.util.List;
import java.util.Set;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.BDDMockito.given;

class CheeringFansMessageTest extends ReportMessageTestBase {

	private void setUpTeams() {
		given(game.getTeamHome().getId()).willReturn("home");
		given(game.getTeamHome().getName()).willReturn("Team home");
		given(game.getTeamHome().getCheerleaders()).willReturn(2);
		given(game.getTeamAway().getId()).willReturn("away");
		given(game.getTeamAway().getName()).willReturn("Team away");
		given(game.getTeamAway().getCheerleaders()).willReturn(1);
	}

	@Test
	public void basicRollsAndTotals() {
		setUpTeams();
		ReportCheeringFans report = new ReportCheeringFans(Collections.emptySet(), 4, 2, Collections.emptySet());
		List<Run> runs = render(new CheeringFansMessage(), report);
		List<String> texts = runs.stream().map(r -> r.text).collect(java.util.stream.Collectors.toList());
		assertTrue(texts.stream().anyMatch(t -> "Cheering Fans Roll Home Team [ 4 ]".equals(t)));
		assertTrue(texts.stream().anyMatch(t -> t != null && t.contains("Rolled 4 + 2 Cheerleaders = 6.")));
		assertTrue(texts.stream().anyMatch(t -> t != null && t.contains("Rolled 2 + 1 Cheerleaders = 3.")));
	}

	@Test
	public void tempAgencyCheerleadersIncluded() {
		setUpTeams();
		given(game.getTurnDataHome().getInducementSet().value(Usage.ADD_CHEERLEADER)).willReturn(3);
		ReportCheeringFans report = new ReportCheeringFans(Collections.emptySet(), 1, 1, Collections.emptySet());
		List<Run> runs = render(new CheeringFansMessage(), report);
		List<String> texts = runs.stream().map(r -> r.text).collect(java.util.stream.Collectors.toList());
		assertTrue(texts.stream().anyMatch(t -> t != null && t.contains("Temp Agency Cheerleaders")));
	}

	@Test
	public void rerolledPrintsMascotMessage() {
		setUpTeams();
		Set<String> rerolled = new HashSet<>();
		rerolled.add("home");
		ReportCheeringFans report = new ReportCheeringFans(Collections.emptySet(), 1, 1, rerolled);
		List<Run> runs = render(new CheeringFansMessage(), report);
		List<String> texts = runs.stream().map(r -> r.text).collect(java.util.stream.Collectors.toList());
		assertTrue(texts.stream().anyMatch(t -> t != null && t.contains("rerolled a natural 1 using their Team Mascot")));
	}

	@Test
	public void teamIdsGainAdditionalAssist() {
		setUpTeams();
		Set<String> teamIds = new HashSet<>();
		teamIds.add("home");
		teamIds.add("away");
		ReportCheeringFans report = new ReportCheeringFans(teamIds, 3, 3, Collections.emptySet());
		List<Run> runs = render(new CheeringFansMessage(), report);
		long count = runs.stream().filter(r -> r.text != null && r.text.contains("gain an additional offensive assist")).count();
		assertEquals(2, count);
	}

	@Test
	public void reportIdIsKickoffCheeringFans() {
		assertEquals(ReportId.KICKOFF_CHEERING_FANS.getKey(), new CheeringFansMessage().getKey());
	}
}
