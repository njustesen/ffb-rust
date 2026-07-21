package com.fumbbl.ffb.client.report.bb2025;

import com.fumbbl.ffb.client.TextStyle;
import com.fumbbl.ffb.client.report.ReportMessageTestBase;
import com.fumbbl.ffb.report.ReportId;
import com.fumbbl.ffb.report.bb2025.ReportTeamCaptainRoll;
import org.junit.jupiter.api.Test;

import java.util.List;
import java.util.Objects;
import java.util.stream.Collectors;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.BDDMockito.given;

class TeamCaptainRollMessageTest extends ReportMessageTestBase {

	private void stubTeams() {
		given(game.getTeamHome().getId()).willReturn("home");
		given(game.getTeamHome().getName()).willReturn("Team home");
		given(game.getTeamAway().getId()).willReturn("away");
		given(game.getTeamAway().getName()).willReturn("Team away");
	}

	@Test
	public void reportIdIsTeamCaptainRoll() {
		assertEquals(ReportId.TEAM_CAPTAIN_ROLL.getKey(), new TeamCaptainRollMessage().getKey());
	}

	@Test
	public void successfulRollSavesRerollWithoutMinimumNote() {
		stubTeams();

		ReportTeamCaptainRoll report = new ReportTeamCaptainRoll("home", 4, 5, true);
		List<Run> runs = render(new TeamCaptainRollMessage(), report);
		List<String> texts = runs.stream().map(r -> r.text).filter(Objects::nonNull).collect(Collectors.toList());

		assertTrue(texts.stream().anyMatch(t -> t.equals("Team Captain Roll [ 5 ]")));
		assertTrue(texts.stream().anyMatch(t -> t.equals(" look to their Team Captain for guidance and save the re-roll.")));
		assertFalse(texts.stream().anyMatch(t -> t.contains("Roll >=")));
	}

	@Test
	public void unsuccessfulRollShowsMinimumNote() {
		stubTeams();

		ReportTeamCaptainRoll report = new ReportTeamCaptainRoll("away", 4, 2, false);
		List<Run> runs = render(new TeamCaptainRollMessage(), report);
		List<String> texts = runs.stream().map(r -> r.text).filter(Objects::nonNull).collect(Collectors.toList());

		assertTrue(texts.stream().anyMatch(t -> t.equals(" look to their Team Captain for guidance but nothing happens.")));
		assertTrue(texts.stream().anyMatch(t -> t.equals("(Roll >= 4 to succeed)")));
	}

	@Test
	public void teamNamePrintedForHomeTeam() {
		stubTeams();

		ReportTeamCaptainRoll report = new ReportTeamCaptainRoll("home", 3, 6, true);
		List<Run> runs = render(new TeamCaptainRollMessage(), report);

		Run teamRun = runs.stream().filter(r -> "Team home".equals(r.text)).findFirst().orElseThrow();
		assertEquals(TextStyle.HOME, teamRun.textStyle);
	}
}
