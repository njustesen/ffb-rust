package com.fumbbl.ffb.client.report.bb2025;

import com.fumbbl.ffb.client.report.ReportMessageTestBase;
import com.fumbbl.ffb.inducement.Usage;
import com.fumbbl.ffb.report.ReportId;
import com.fumbbl.ffb.report.mixed.ReportKickoffExtraReRoll;
import org.junit.jupiter.api.Test;

import java.util.List;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.BDDMockito.given;

class KickoffExtraReRollMessageTest extends ReportMessageTestBase {

	private void stubTeams() {
		given(game.getTeamHome().getId()).willReturn("home");
		given(game.getTeamHome().getName()).willReturn("Team home");
		given(game.getTeamHome().getAssistantCoaches()).willReturn(2);
		given(game.getTeamAway().getId()).willReturn("away");
		given(game.getTeamAway().getName()).willReturn("Team away");
		given(game.getTeamAway().getAssistantCoaches()).willReturn(1);
		given(game.getTurnDataHome().getInducementSet().value(Usage.ADD_COACH)).willReturn(0);
		given(game.getTurnDataAway().getInducementSet().value(Usage.ADD_COACH)).willReturn(0);
	}

	@Test
	public void bothTeamsGainRerollWhenNoTeamId() {
		stubTeams();
		ReportKickoffExtraReRoll report = new ReportKickoffExtraReRoll(4, 3, null);
		List<Run> runs = render(new KickoffExtraReRollMessage(), report);
		List<String> texts = runs.stream().map(r -> r.text).collect(java.util.stream.Collectors.toList());
		assertTrue(texts.contains("Both teams gain a Re-Roll only available for this drive."));
		assertTrue(texts.stream().anyMatch(t -> "Rolled 4 + 2 Assistant Coaches = 6.".equals(t)));
		assertTrue(texts.stream().anyMatch(t -> "Rolled 3 + 1 Assistant Coaches = 4.".equals(t)));
	}

	@Test
	public void homeTeamGainsRerollWhenTeamIdMatchesHome() {
		stubTeams();
		ReportKickoffExtraReRoll report = new ReportKickoffExtraReRoll(4, 3, "home");
		List<Run> runs = render(new KickoffExtraReRollMessage(), report);
		List<String> texts = runs.stream().map(r -> r.text).collect(java.util.stream.Collectors.toList());
		assertTrue(texts.contains("Team home"));
		assertTrue(texts.contains(" gains a Re-Roll only available for this drive."));
	}

	@Test
	public void awayTeamGainsRerollWhenTeamIdMatchesAway() {
		stubTeams();
		ReportKickoffExtraReRoll report = new ReportKickoffExtraReRoll(1, 1, "away");
		List<Run> runs = render(new KickoffExtraReRollMessage(), report);
		List<String> texts = runs.stream().map(r -> r.text).collect(java.util.stream.Collectors.toList());
		assertTrue(texts.contains("Team away"));
	}

	@Test
	public void partTimeAssistantCoachesIncludedWhenPresent() {
		stubTeams();
		given(game.getTurnDataHome().getInducementSet().value(Usage.ADD_COACH)).willReturn(1);
		ReportKickoffExtraReRoll report = new ReportKickoffExtraReRoll(4, 3, null);
		List<Run> runs = render(new KickoffExtraReRollMessage(), report);
		List<String> texts = runs.stream().map(r -> r.text).collect(java.util.stream.Collectors.toList());
		assertTrue(texts.stream().anyMatch(t -> t != null && t.startsWith("Rolled 4 + 2 Assistant Coaches")));
		assertTrue(texts.stream().anyMatch(t -> "Rolled 4 + 2 Assistant Coaches + 1 Part-time Assistant Coaches = 7.".equals(t)));
	}

	@Test
	public void reportIdIsKickoffExtraReRoll() {
		assertEquals(ReportId.KICKOFF_EXTRA_RE_ROLL.getKey(), new KickoffExtraReRollMessage().getKey());
	}
}
