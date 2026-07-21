package com.fumbbl.ffb.client.report;

import com.fumbbl.ffb.client.ParagraphStyle;
import com.fumbbl.ffb.client.TextStyle;
import com.fumbbl.ffb.report.ReportTimeoutEnforced;
import org.junit.jupiter.api.Test;

import java.util.List;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.mockito.BDDMockito.given;

class TimeoutEnforcedMessageTest extends ReportMessageTestBase {

	@Test
	public void homeCoachUsesHomeBold() {
		given(game.getTeamHome().getCoach()).willReturn("HomeCoach");

		ReportTimeoutEnforced report = new ReportTimeoutEnforced("HomeCoach");
		List<Run> runs = render(new TimeoutEnforcedMessage(), report);

		assertEquals("Coach HomeCoach forces a Timeout.", runs.get(0).text);
		assertEquals(TextStyle.HOME_BOLD, runs.get(0).textStyle);
		assertEquals(ParagraphStyle.SPACE_ABOVE, runs.get(0).paragraphStyle);
	}

	@Test
	public void awayCoachUsesAwayBold() {
		given(game.getTeamHome().getCoach()).willReturn("HomeCoach");

		ReportTimeoutEnforced report = new ReportTimeoutEnforced("AwayCoach");
		List<Run> runs = render(new TimeoutEnforcedMessage(), report);

		assertEquals(TextStyle.AWAY_BOLD, runs.get(0).textStyle);
	}

	@Test
	public void unknownCoachFallsThroughToAwayBold() {
		given(game.getTeamHome().getCoach()).willReturn("HomeCoach");

		ReportTimeoutEnforced report = new ReportTimeoutEnforced("SomeoneElse");
		List<Run> runs = render(new TimeoutEnforcedMessage(), report);

		assertEquals(TextStyle.AWAY_BOLD, runs.get(0).textStyle);
	}

	@Test
	public void secondLineUsesSpaceBelowAndNoneStyle() {
		given(game.getTeamHome().getCoach()).willReturn("HomeCoach");

		ReportTimeoutEnforced report = new ReportTimeoutEnforced("HomeCoach");
		List<Run> runs = render(new TimeoutEnforcedMessage(), report);

		assertEquals("The turn will end after the Acting Player has finished moving.", runs.get(2).text);
		assertEquals(ParagraphStyle.SPACE_BELOW, runs.get(2).paragraphStyle);
		assertEquals(TextStyle.NONE, runs.get(2).textStyle);
	}
}
