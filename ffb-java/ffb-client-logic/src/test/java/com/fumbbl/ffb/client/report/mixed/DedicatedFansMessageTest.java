package com.fumbbl.ffb.client.report.mixed;

import com.fumbbl.ffb.client.report.ReportMessageTestBase;
import com.fumbbl.ffb.model.Team;
import com.fumbbl.ffb.report.mixed.ReportDedicatedFans;
import org.junit.jupiter.api.Test;

import java.util.List;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.mockito.BDDMockito.given;

class DedicatedFansMessageTest extends ReportMessageTestBase {

	@Test
	public void rendersGainForPositiveModifierHome() {
		given(game.getTeamHome().getName()).willReturn("Team home");
		given(game.getTeamAway().getName()).willReturn("Team away");

		ReportDedicatedFans report = new ReportDedicatedFans(3, 1, 0, 0, null, false);
		List<Run> runs = render(new DedicatedFansMessage(), report);

		assertEquals("Dedicated Fans Roll [ 3 ]", runs.get(0).text);
		assertEquals("Team home", runs.get(2).text);
		assertEquals(" gain 1 Dedicated Fan.", runs.get(3).text);
	}

	@Test
	public void rendersLoseForNegativeModifierAndPlural() {
		given(game.getTeamHome().getName()).willReturn("Team home");
		given(game.getTeamAway().getName()).willReturn("Team away");

		ReportDedicatedFans report = new ReportDedicatedFans(0, 0, 5, -2, null, false);
		List<Run> runs = render(new DedicatedFansMessage(), report);

		// home roll is 0 -> no roll line, only team name + text.
		assertEquals("Team home", runs.get(0).text);
		assertEquals(" keep their Dedicated Fans.", runs.get(1).text);
		// away roll is 5 -> roll line emitted.
		assertEquals("Dedicated Fans Roll [ 5 ]", runs.get(3).text);
		assertEquals(" lose 2 Dedicated Fans.", runs.get(6).text);
	}

	@Test
	public void rendersConcedingSuffixWhenConcededAndNonzeroModifier() {
		Team homeTeam = game.getTeamHome();
		given(homeTeam.getName()).willReturn("Team home");
		given(game.getTeamAway().getName()).willReturn("Team away");
		given(game.getTeamById("home")).willReturn(homeTeam);

		ReportDedicatedFans report = new ReportDedicatedFans(2, -1, 0, 0, "home", true);
		List<Run> runs = render(new DedicatedFansMessage(), report);

		assertEquals(" lose 1 Dedicated Fan due to conceding.", runs.get(3).text);
	}

	@Test
	public void noConcedingSuffixWhenModifierZeroEvenIfConceded() {
		Team homeTeam = game.getTeamHome();
		given(homeTeam.getName()).willReturn("Team home");
		given(game.getTeamAway().getName()).willReturn("Team away");
		given(game.getTeamById("home")).willReturn(homeTeam);

		ReportDedicatedFans report = new ReportDedicatedFans(0, 0, 0, 0, "home", true);
		List<Run> runs = render(new DedicatedFansMessage(), report);

		assertEquals(" keep their Dedicated Fans.", runs.get(1).text);
	}
}
