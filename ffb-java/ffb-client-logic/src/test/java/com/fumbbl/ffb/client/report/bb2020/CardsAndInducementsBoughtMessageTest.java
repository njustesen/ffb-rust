package com.fumbbl.ffb.client.report.bb2020;

import com.fumbbl.ffb.client.TextStyle;
import com.fumbbl.ffb.client.report.ReportMessageTestBase;
import com.fumbbl.ffb.report.bb2020.ReportCardsAndInducementsBought;
import org.junit.jupiter.api.Test;

import java.util.List;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.BDDMockito.given;

class CardsAndInducementsBoughtMessageTest extends ReportMessageTestBase {

	@Test
	public void firstReportPrintsBuyInducementsHeader() {
		given(game.getTeamHome().getId()).willReturn("home");
		given(game.getTeamHome().getName()).willReturn("Home Team");

		ReportCardsAndInducementsBought report = new ReportCardsAndInducementsBought("home", 0, 0, 0, 0, 0, 1_000_000);
		List<Run> runs = render(new CardsAndInducementsBoughtMessage(), report);

		assertTrue(runs.stream().anyMatch(r -> "Buy Inducements".equals(r.text)));
		assertTrue(statusReport.inducementsBoughtReportReceived);
	}

	@Test
	public void secondReportDoesNotRepeatHeader() {
		statusReport.inducementsBoughtReportReceived = true;
		given(game.getTeamHome().getId()).willReturn("home");
		given(game.getTeamAway().getName()).willReturn("Away Team");

		ReportCardsAndInducementsBought report = new ReportCardsAndInducementsBought("away", 1, 0, 0, 0, 100_000, 1_100_000);
		List<Run> runs = render(new CardsAndInducementsBoughtMessage(), report);

		assertFalse(runs.stream().anyMatch(r -> "Buy Inducements".equals(r.text)));
	}

	@Test
	public void noItemsBoughtPrintsNoInducements() {
		given(game.getTeamHome().getId()).willReturn("home");
		given(game.getTeamHome().getName()).willReturn("Home Team");

		ReportCardsAndInducementsBought report = new ReportCardsAndInducementsBought("home", 0, 0, 0, 0, 0, 1_000_000);
		List<Run> runs = render(new CardsAndInducementsBoughtMessage(), report);

		assertTrue(runs.stream().anyMatch(r -> r.text != null && r.text.contains("no Inducements.")));
	}

	@Test
	public void singleCardAndMultipleStarsUseCorrectPluralization() {
		given(game.getTeamHome().getId()).willReturn("home");
		given(game.getTeamHome().getName()).willReturn("Home Team");

		ReportCardsAndInducementsBought report = new ReportCardsAndInducementsBought("home", 1, 0, 2, 0, 100_000, 1_200_000);
		List<Run> runs = render(new CardsAndInducementsBoughtMessage(), report);

		assertTrue(runs.stream().anyMatch(r -> r.text != null && r.text.contains("1 Card") && r.text.contains("2 Stars")));
		assertTrue(runs.stream().anyMatch(r -> r.text != null && r.text.contains("100,000 gold total increasing their Team Value to 1,200,000")));
	}

	@Test
	public void awayTeamUsesAwayStyle() {
		given(game.getTeamHome().getId()).willReturn("home");
		given(game.getTeamAway().getName()).willReturn("Away Team");

		ReportCardsAndInducementsBought report = new ReportCardsAndInducementsBought("away", 0, 1, 0, 1, 50_000, 900_000);
		List<Run> runs = render(new CardsAndInducementsBoughtMessage(), report);

		Run teamRun = runs.stream().filter(r -> "Away Team".equals(r.text)).findFirst().orElseThrow();
		assertEquals(TextStyle.AWAY, teamRun.textStyle);
		assertTrue(runs.stream().anyMatch(r -> r.text != null && r.text.contains("1 Inducement") && r.text.contains("1 Mercenary")));
	}
}
