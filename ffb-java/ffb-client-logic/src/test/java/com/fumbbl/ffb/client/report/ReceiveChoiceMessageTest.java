package com.fumbbl.ffb.client.report;

import com.fumbbl.ffb.client.ParagraphStyle;
import com.fumbbl.ffb.report.ReportReceiveChoice;
import org.junit.jupiter.api.Test;

import java.util.List;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.mockito.BDDMockito.given;

class ReceiveChoiceMessageTest extends ReportMessageTestBase {

	@Test
	public void renderReceivingHomeTeam() {
		given(game.getTeamHome().getId()).willReturn("home");
		given(game.getTeamHome().getName()).willReturn("home");

		ReportReceiveChoice report = new ReportReceiveChoice("home", true);
		List<Run> runs = render(new ReceiveChoiceMessage(), report);

		assertEquals("Team ", runs.get(0).text);
		assertEquals("home", runs.get(1).text);
		assertEquals(" is receiving.", runs.get(2).text);
	}

	@Test
	public void renderKickingAwayTeam() {
		given(game.getTeamHome().getId()).willReturn("home");
		given(game.getTeamAway().getName()).willReturn("away");

		ReportReceiveChoice report = new ReportReceiveChoice("away", false);
		List<Run> runs = render(new ReceiveChoiceMessage(), report);

		assertEquals(" is kicking.", runs.get(2).text);
	}

	@Test
	public void renderUsesIndentPlusOne() {
		given(game.getTeamHome().getId()).willReturn("home");
		given(game.getTeamHome().getName()).willReturn("home");
		statusReport.setIndent(2);

		ReportReceiveChoice report = new ReportReceiveChoice("home", true);
		List<Run> runs = render(new ReceiveChoiceMessage(), report);

		assertEquals(ParagraphStyle.INDENT_3, runs.get(0).paragraphStyle);
	}
}
