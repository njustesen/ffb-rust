package com.fumbbl.ffb.client.report.bb2016;

import com.fumbbl.ffb.client.report.ReportMessageTestBase;
import com.fumbbl.ffb.report.bb2016.ReportInducementsBought;
import org.junit.jupiter.api.Test;

import java.util.List;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.BDDMockito.given;

class InducementsBoughtMessageTest extends ReportMessageTestBase {

	@Test
	public void getKeyIsInducementsBought() {
		assertEquals("inducementsBought", new InducementsBoughtMessage().getKey());
	}

	@Test
	public void noInducementsBought() {
		given(game.getTeamHome().getId()).willReturn("home");

		ReportInducementsBought report = new ReportInducementsBought("home", 0, 0, 0, 0);
		List<Run> runs = render(new InducementsBoughtMessage(), report);

		assertTrue(runs.stream().anyMatch(r -> " buys no Inducements.".equals(r.text)));
	}

	@Test
	public void enumeratesMultipleItems() {
		given(game.getTeamHome().getId()).willReturn("home");

		ReportInducementsBought report = new ReportInducementsBought("away", 2, 1, 0, 50000);
		List<Run> runs = render(new InducementsBoughtMessage(), report);

		assertTrue(runs.stream().anyMatch(r -> " buys 2 Inducements and 1 Star for 50,000 gold total.".equals(r.text)));
	}

	@Test
	public void secondCallSkipsHeader() {
		statusReport.inducementsBoughtReportReceived = true;
		given(game.getTeamHome().getId()).willReturn("home");

		ReportInducementsBought report = new ReportInducementsBought("home", 0, 0, 0, 0);
		List<Run> runs = render(new InducementsBoughtMessage(), report);

		assertFalse(runs.stream().anyMatch(r -> "Buy Inducements".equals(r.text)));
	}
}
