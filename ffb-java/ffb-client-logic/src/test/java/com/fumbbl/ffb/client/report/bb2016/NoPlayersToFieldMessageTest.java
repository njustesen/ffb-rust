package com.fumbbl.ffb.client.report.bb2016;

import com.fumbbl.ffb.client.report.ReportMessageTestBase;
import com.fumbbl.ffb.report.bb2016.ReportNoPlayersToField;
import org.junit.jupiter.api.Test;

import java.util.List;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.BDDMockito.given;

class NoPlayersToFieldMessageTest extends ReportMessageTestBase {

	@Test
	public void getKeyIsNoPlayersToField() {
		assertEquals("noPlayersToField", new NoPlayersToFieldMessage().getKey());
	}

	@Test
	public void teamIdProvidedReportsTouchdownAward() {
		given(game.getTeamHome().getId()).willReturn("home");
		given(game.getTeamHome().getName()).willReturn("Team home");

		ReportNoPlayersToField report = new ReportNoPlayersToField("home");
		List<Run> runs = render(new NoPlayersToFieldMessage(), report);

		assertTrue(runs.stream().anyMatch(r -> "Team home can field no players.".equals(r.text)));
		assertTrue(runs.stream().anyMatch(r -> "The opposing team is awarded a touchdown.".equals(r.text)));
	}

	@Test
	public void noTeamIdReportsBothTeams() {
		ReportNoPlayersToField report = new ReportNoPlayersToField("");
		List<Run> runs = render(new NoPlayersToFieldMessage(), report);

		assertTrue(runs.stream().anyMatch(r -> "Both teams can field no players.".equals(r.text)));
		assertFalse(runs.stream().anyMatch(r -> "The opposing team is awarded a touchdown.".equals(r.text)));
	}

	@Test
	public void alwaysReportsTurnAdvance() {
		given(game.getTeamHome().getId()).willReturn("home");
		given(game.getTeamAway().getName()).willReturn("Team away");

		ReportNoPlayersToField report = new ReportNoPlayersToField("away");
		List<Run> runs = render(new NoPlayersToFieldMessage(), report);

		assertTrue(runs.stream().anyMatch(r -> "The turn counter is advanced 2 steps.".equals(r.text)));
	}
}
