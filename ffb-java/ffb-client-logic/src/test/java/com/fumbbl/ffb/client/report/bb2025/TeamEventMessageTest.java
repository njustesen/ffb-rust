package com.fumbbl.ffb.client.report.bb2025;

import com.fumbbl.ffb.client.TextStyle;
import com.fumbbl.ffb.client.report.ReportMessageTestBase;
import com.fumbbl.ffb.report.ReportId;
import com.fumbbl.ffb.report.bb2025.ReportTeamEvent;
import org.junit.jupiter.api.Test;

import java.util.List;
import java.util.Objects;
import java.util.stream.Collectors;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.BDDMockito.given;

class TeamEventMessageTest extends ReportMessageTestBase {

	private void stubTeams() {
		given(game.getTeamHome().getId()).willReturn("home");
		given(game.getTeamHome().getName()).willReturn("Team home");
		given(game.getTeamAway().getId()).willReturn("away");
		given(game.getTeamAway().getName()).willReturn("Team away");
	}

	@Test
	public void reportIdIsTeamEvent() {
		assertEquals(ReportId.TEAM_EVENT.getKey(), new TeamEventMessage().getKey());
	}

	@Test
	public void rendersHomeTeamNameAndMessage() {
		stubTeams();

		ReportTeamEvent report = new ReportTeamEvent("home", "Player banned!");
		List<Run> runs = render(new TeamEventMessage(), report);
		List<String> texts = runs.stream().map(r -> r.text).filter(Objects::nonNull).collect(Collectors.toList());

		assertTrue(texts.stream().anyMatch(t -> t.equals("Team home")));
		assertTrue(texts.stream().anyMatch(t -> t.equals(" Player banned!")));
	}

	@Test
	public void rendersAwayTeamNameAndMessage() {
		stubTeams();

		ReportTeamEvent report = new ReportTeamEvent("away", "Mascot injured!");
		List<Run> runs = render(new TeamEventMessage(), report);
		List<String> texts = runs.stream().map(r -> r.text).filter(Objects::nonNull).collect(Collectors.toList());

		assertTrue(texts.stream().anyMatch(t -> t.equals("Team away")));
		assertTrue(texts.stream().anyMatch(t -> t.equals(" Mascot injured!")));
	}

	@Test
	public void messageUsesNoneTextStyleAtIndentPlusOne() {
		stubTeams();
		statusReport.setIndent(1);

		ReportTeamEvent report = new ReportTeamEvent("home", "Test");
		List<Run> runs = render(new TeamEventMessage(), report);

		Run run = runs.stream().filter(r -> " Test".equals(r.text)).findFirst().orElseThrow();
		assertEquals(TextStyle.NONE, run.textStyle);
	}
}
