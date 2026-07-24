package com.fumbbl.ffb.client.report.mixed;

import com.fumbbl.ffb.client.TextStyle;
import com.fumbbl.ffb.client.report.ReportMessageTestBase;
import com.fumbbl.ffb.report.mixed.ReportFreePettyCash;
import com.fumbbl.ffb.util.StringTool;
import org.junit.jupiter.api.Test;

import java.util.List;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.BDDMockito.given;

class FreePettyCashMessageTest extends ReportMessageTestBase {

	@Test
	public void rendersHomeTeamPettyCash() {
		given(game.getTeamHome().getId()).willReturn("home");
		given(game.getTeamHome().getName()).willReturn("Team home");

		ReportFreePettyCash report = new ReportFreePettyCash("home", 50000);
		List<Run> runs = render(new FreePettyCashMessage(), report);

		assertEquals("Assigning Petty Cash", runs.get(0).text);
		assertEquals(TextStyle.BOLD, runs.get(0).textStyle);
		assertEquals("Team ", runs.get(2).text);
		assertEquals("Team home", runs.get(3).text);
		assertEquals(TextStyle.HOME, runs.get(3).textStyle);
		assertEquals(" receives 50,000 gold as petty cash from being the underdog before adding inducements.",
			runs.get(4).text);
	}

	@Test
	public void rendersAwayTeamPettyCash() {
		given(game.getTeamHome().getId()).willReturn("home");
		given(game.getTeamAway().getName()).willReturn("Team away");

		ReportFreePettyCash report = new ReportFreePettyCash("away", 120000);
		List<Run> runs = render(new FreePettyCashMessage(), report);

		assertEquals("Team away", runs.get(3).text);
		assertEquals(TextStyle.AWAY, runs.get(3).textStyle);
		assertTrue(runs.get(4).text.contains("120,000"));
	}

	@Test
	public void fallsBackToAwayWhenTeamIdMissing() {
		given(game.getTeamHome().getId()).willReturn("home");
		given(game.getTeamAway().getName()).willReturn("Team away");

		ReportFreePettyCash report = new ReportFreePettyCash(null, 1000);
		List<Run> runs = render(new FreePettyCashMessage(), report);

		assertEquals("Team away", runs.get(3).text);
	}

	// rust: format_thousands_matches_java_examples
	@Test
	public void formatThousandsMatchesJavaExamples() {
		assertEquals("2,130,000", StringTool.formatThousands(2130000));
		assertEquals("50,000", StringTool.formatThousands(50000));
		assertEquals("0", StringTool.formatThousands(0));
		assertEquals("100", StringTool.formatThousands(100));
	}
}
