package com.fumbbl.ffb.client.report.mixed;

import com.fumbbl.ffb.client.TextStyle;
import com.fumbbl.ffb.client.report.ReportMessageTestBase;
import com.fumbbl.ffb.report.mixed.ReportKickoffTimeout;
import org.junit.jupiter.api.Test;

import java.util.List;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.mockito.BDDMockito.given;

class KickoffTimeoutMessageTest extends ReportMessageTestBase {

	@Test
	public void homePlayingNegativeModifier() {
		given(game.isHomePlaying()).willReturn(true);
		given(game.getTeamHome().getName()).willReturn("Team home");

		// ReportKickoffTimeout ctor is (turnNumber, turnModifier); turn 5, modifier -2.
		ReportKickoffTimeout report = new ReportKickoffTimeout(5, -2);
		List<Run> runs = render(new KickoffTimeoutMessage(), report);

		assertEquals("Timeout in turn 5 of ", runs.get(0).text);
		assertEquals("Team home", runs.get(1).text);
		assertEquals(TextStyle.HOME, runs.get(1).textStyle);
		assertEquals("The referee adjusts the clock back.", runs.get(3).text);
		assertEquals("Turn Counter is moved 2 step backward.", runs.get(5).text);
	}

	@Test
	public void awayPlayingPositiveModifier() {
		given(game.isHomePlaying()).willReturn(false);
		given(game.getTeamAway().getName()).willReturn("Team away");

		// ReportKickoffTimeout ctor is (turnNumber, turnModifier); turn 1, modifier 3.
		ReportKickoffTimeout report = new ReportKickoffTimeout(1, 3);
		List<Run> runs = render(new KickoffTimeoutMessage(), report);

		assertEquals("Team away", runs.get(1).text);
		assertEquals(TextStyle.AWAY, runs.get(1).textStyle);
		assertEquals("The referee does not stop the clock.", runs.get(3).text);
		assertEquals("Turn Counter is moved 3 step forward.", runs.get(5).text);
	}
}
