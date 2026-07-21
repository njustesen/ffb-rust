package com.fumbbl.ffb.client.report.bb2016;

import com.fumbbl.ffb.client.report.ReportMessageTestBase;
import com.fumbbl.ffb.report.bb2016.ReportFanFactorRollPostMatch;
import org.junit.jupiter.api.Test;

import java.util.List;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.mockito.BDDMockito.given;

class FanFactorRollPostMatchMessageTest extends ReportMessageTestBase {

	@Test
	public void getKeyIsFanFactorRollPostMatch() {
		assertEquals("fanFactorRoll", new FanFactorRollPostMatchMessage().getKey());
	}

	@Test
	public void positiveModifierReportsWinSomeNewFans() {
		given(game.getTeamHome().getFanFactor()).willReturn(5);
		given(game.getTeamAway().getFanFactor()).willReturn(3);

		ReportFanFactorRollPostMatch report = new ReportFanFactorRollPostMatch(new int[]{4}, 1, new int[]{2}, -1);
		List<Run> runs = render(new FanFactorRollPostMatchMessage(), report);

		assertEquals(1, runs.stream().filter(r -> " win some new fans.".equals(r.text)).count());
		assertEquals(1, runs.stream().filter(r -> " lose some fans.".equals(r.text)).count());
	}

	@Test
	public void zeroModifierKeepsFans() {
		given(game.getTeamHome().getFanFactor()).willReturn(5);
		given(game.getTeamAway().getFanFactor()).willReturn(3);

		ReportFanFactorRollPostMatch report = new ReportFanFactorRollPostMatch(new int[]{4}, 0, new int[]{2}, 0);
		List<Run> runs = render(new FanFactorRollPostMatchMessage(), report);

		assertEquals(2, runs.stream().filter(r -> " keep their fans.".equals(r.text)).count());
	}

	@Test
	public void concessionWhenRollEmpty() {
		given(game.getTeamHome().getFanFactor()).willReturn(5);
		given(game.getTeamAway().getFanFactor()).willReturn(3);

		ReportFanFactorRollPostMatch report = new ReportFanFactorRollPostMatch(new int[]{}, 0, new int[]{2}, 0);
		List<Run> runs = render(new FanFactorRollPostMatchMessage(), report);

		assertEquals("Fan Factor: Concession of Home Team", runs.get(0).text);
	}
}
