package com.fumbbl.ffb.client.report.bb2016;

import com.fumbbl.ffb.client.report.ReportMessageTestBase;
import com.fumbbl.ffb.report.bb2016.ReportSpectators;
import org.junit.jupiter.api.Test;

import java.util.List;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.BDDMockito.given;

class SpectatorsMessageTest extends ReportMessageTestBase {

	@Test
	public void getKeyIsSpectators() {
		assertEquals("spectators", new SpectatorsMessage().getKey());
	}

	@Test
	public void homeFameAdvantageReportsHomeBold() {
		given(game.getTeamHome().getName()).willReturn("Team home");

		ReportSpectators report = new ReportSpectators(new int[]{4, 4}, 20000, 2, new int[]{2, 3}, 10000, 0);
		List<Run> runs = render(new SpectatorsMessage(), report);

		assertTrue(runs.stream().anyMatch(r -> "Team Team home have the whole audience with them (FAME +2)!".equals(r.text)));
	}

	@Test
	public void equalFameReportsEqualMessage() {
		ReportSpectators report = new ReportSpectators(new int[]{4, 4}, 20000, 1, new int[]{2, 3}, 10000, 1);
		List<Run> runs = render(new SpectatorsMessage(), report);

		assertTrue(runs.stream().anyMatch(r -> "Both teams have equal fan support (FAME 0).".equals(r.text)));
	}

	@Test
	public void reportsSpectatorCounts() {
		ReportSpectators report = new ReportSpectators(new int[]{4, 4}, 20000, 0, new int[]{2, 3}, 10000, 0);
		List<Run> runs = render(new SpectatorsMessage(), report);

		assertTrue(runs.stream().anyMatch(r -> "20,000 fans have come to support ".equals(r.text)));
	}
}
