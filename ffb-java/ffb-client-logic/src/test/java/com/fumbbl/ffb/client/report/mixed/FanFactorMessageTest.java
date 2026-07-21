package com.fumbbl.ffb.client.report.mixed;

import com.fumbbl.ffb.client.TextStyle;
import com.fumbbl.ffb.client.report.ReportMessageTestBase;
import com.fumbbl.ffb.report.mixed.ReportFanFactor;
import org.junit.jupiter.api.Test;

import java.util.List;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.mockito.BDDMockito.given;

class FanFactorMessageTest extends ReportMessageTestBase {

	@Test
	public void rendersHomeTeamFanFactor() {
		given(game.getTeamHome().getId()).willReturn("home");
		given(game.getTeamHome().getName()).willReturn("Team home");

		ReportFanFactor report = new ReportFanFactor("home", 3, 2);
		List<Run> runs = render(new FanFactorMessage(), report);

		assertEquals("Fan Factor Roll [3]", runs.get(0).text);
		assertEquals("Team ", runs.get(2).text);
		assertEquals("Team home", runs.get(3).text);
		assertEquals(TextStyle.HOME, runs.get(3).textStyle);
		assertEquals(" has 5k fans behind them (2k Dedicated Fans and 3k fair-weather fans)", runs.get(4).text);
	}

	@Test
	public void rendersAwayTeamFanFactor() {
		given(game.getTeamHome().getId()).willReturn("home");
		given(game.getTeamAway().getName()).willReturn("Team away");

		ReportFanFactor report = new ReportFanFactor("away", 1, 0);
		List<Run> runs = render(new FanFactorMessage(), report);

		assertEquals("Team away", runs.get(3).text);
		assertEquals(TextStyle.AWAY, runs.get(3).textStyle);
	}

	@Test
	public void fallsBackToAwayWhenTeamIdMissing() {
		given(game.getTeamHome().getId()).willReturn("home");
		given(game.getTeamAway().getName()).willReturn("Team away");

		ReportFanFactor report = new ReportFanFactor(null, 4, 1);
		List<Run> runs = render(new FanFactorMessage(), report);

		assertEquals("Team away", runs.get(3).text);
	}
}
