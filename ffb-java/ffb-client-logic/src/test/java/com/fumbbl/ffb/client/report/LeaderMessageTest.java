package com.fumbbl.ffb.client.report;

import com.fumbbl.ffb.LeaderState;
import com.fumbbl.ffb.client.ParagraphStyle;
import com.fumbbl.ffb.report.ReportLeader;
import org.junit.jupiter.api.Test;

import java.util.List;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.BDDMockito.given;

class LeaderMessageTest extends ReportMessageTestBase {

	private void stubTeams() {
		given(game.getTeamHome().getId()).willReturn("home");
		given(game.getTeamHome().getName()).willReturn("home");
		given(game.getTeamAway().getId()).willReturn("away");
		given(game.getTeamAway().getName()).willReturn("away");
	}

	@Test
	public void availablePrintsTeamNameThenGainMessage() {
		stubTeams();
		ReportLeader report = new ReportLeader("home", LeaderState.AVAILABLE);
		List<Run> runs = render(new LeaderMessage(), report);

		assertEquals("home", runs.get(0).text);
		assertTrue(runs.stream().anyMatch(r -> " gain a Leader re-roll.".equals(r.text)));
		assertTrue(runs.stream().anyMatch(r -> ".".equals(r.text)));
	}

	@Test
	public void usedPrintsRemovedMessageBeforeTeamName() {
		stubTeams();
		ReportLeader report = new ReportLeader("away", LeaderState.USED);
		List<Run> runs = render(new LeaderMessage(), report);

		assertEquals("Leader re-roll removed from ", runs.get(0).text);
		assertTrue(runs.stream().anyMatch(r -> "away".equals(r.text)));
	}

	@Test
	public void indentResetToZero() {
		stubTeams();
		statusReport.setIndent(5);
		ReportLeader report = new ReportLeader("home", LeaderState.AVAILABLE);
		List<Run> runs = render(new LeaderMessage(), report);

		// render prints at indent getIndent() + 1 == 1 after setIndent(0)
		assertEquals(ParagraphStyle.INDENT_1, runs.get(0).paragraphStyle);
	}
}
