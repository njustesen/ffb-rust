package com.fumbbl.ffb.client.report.bb2016;

import com.fumbbl.ffb.client.report.ReportMessageTestBase;
import com.fumbbl.ffb.report.bb2016.ReportCardsBought;
import org.junit.jupiter.api.Test;

import java.util.List;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.BDDMockito.given;

class CardsBoughtMessageTest extends ReportMessageTestBase {

	@Test
	public void getKeyIsCardsBought() {
		assertEquals("cardsBought", new CardsBoughtMessage().getKey());
	}

	@Test
	public void firstReportPrintsHeaderAndSetsFlag() {
		given(game.getTeamHome().getId()).willReturn("home");

		ReportCardsBought report = new ReportCardsBought("home", 2, 20000);
		List<Run> runs = render(new CardsBoughtMessage(), report);

		assertTrue(statusReport.fCardsBoughtReportReceived);
		assertEquals("Buy Cards", runs.get(0).text);
	}

	@Test
	public void secondReportSkipsHeader() {
		statusReport.fCardsBoughtReportReceived = true;
		given(game.getTeamHome().getId()).willReturn("home");

		ReportCardsBought report = new ReportCardsBought("away", 0, 0);
		List<Run> runs = render(new CardsBoughtMessage(), report);

		assertFalse(runs.stream().anyMatch(r -> "Buy Cards".equals(r.text)));
		assertTrue(runs.stream().anyMatch(r -> " buys no Cards.".equals(r.text)));
	}

	@Test
	public void singleCardUsesSingular() {
		given(game.getTeamHome().getId()).willReturn("home");

		ReportCardsBought report = new ReportCardsBought("home", 1, 10000);
		List<Run> runs = render(new CardsBoughtMessage(), report);

		assertTrue(runs.stream().anyMatch(r -> " buys 1 Card for 10,000 gold total.".equals(r.text)));
	}
}
