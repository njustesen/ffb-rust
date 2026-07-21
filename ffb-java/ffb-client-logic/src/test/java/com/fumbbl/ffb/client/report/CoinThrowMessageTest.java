package com.fumbbl.ffb.client.report;

import com.fumbbl.ffb.client.ParagraphStyle;
import com.fumbbl.ffb.client.TextStyle;
import com.fumbbl.ffb.report.ReportCoinThrow;
import org.junit.jupiter.api.Test;

import java.util.List;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.BDDMockito.given;

class CoinThrowMessageTest extends ReportMessageTestBase {

	@Test
	public void homeCoachUsesHomeStyle() {
		given(game.getTeamHome().getCoach()).willReturn("CoachA");

		ReportCoinThrow report = new ReportCoinThrow(true, "CoachA", false);
		List<Run> runs = render(new CoinThrowMessage(), report);

		Run coachRun = runs.stream().filter(r -> "CoachA".equals(r.text)).findFirst().orElseThrow();
		assertEquals(TextStyle.HOME, coachRun.textStyle);
	}

	@Test
	public void awayCoachUsesAwayStyle() {
		given(game.getTeamHome().getCoach()).willReturn("CoachA");

		ReportCoinThrow report = new ReportCoinThrow(true, "CoachB", false);
		List<Run> runs = render(new CoinThrowMessage(), report);

		Run coachRun = runs.stream().filter(r -> "CoachB".equals(r.text)).findFirst().orElseThrow();
		assertEquals(TextStyle.AWAY, coachRun.textStyle);
	}

	@Test
	public void coinChoiceAndThrowText() {
		given(game.getTeamHome().getCoach()).willReturn("CoachA");

		ReportCoinThrow report = new ReportCoinThrow(false, "CoachA", true);
		List<Run> runs = render(new CoinThrowMessage(), report);

		assertTrue(runs.stream().anyMatch(r -> " chooses HEADS.".equals(r.text)));
		assertTrue(runs.stream().anyMatch(r -> "Coin throw is TAILS.".equals(r.text)));
	}

	@Test
	public void resetsIndentToZero() {
		statusReport.setIndent(3);
		given(game.getTeamHome().getCoach()).willReturn("CoachA");

		ReportCoinThrow report = new ReportCoinThrow(true, "CoachA", false);
		List<Run> runs = render(new CoinThrowMessage(), report);

		assertEquals(ParagraphStyle.INDENT_0, runs.get(0).paragraphStyle);
	}

	@Test
	public void reportIdIsCoinThrow() {
		assertEquals("coinThrow", new CoinThrowMessage().getKey());
	}
}
