package com.fumbbl.ffb.client.report;

import com.fumbbl.ffb.client.TextStyle;
import com.fumbbl.ffb.report.ReportMasterChefRoll;

import org.junit.jupiter.api.Test;

import java.util.List;

import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.BDDMockito.given;

class MasterChefRollMessageTest extends ReportMessageTestBase {

	@Test
	public void zeroStolenSaysNoRerolls() {
		given(game.getTeamHome().getId()).willReturn("home");
		given(game.getTeamAway().getId()).willReturn("away");
		given(game.getTeamAway().getName()).willReturn("away");

		ReportMasterChefRoll report = new ReportMasterChefRoll("home", new int[]{1, 2, 3}, 0);
		List<Run> runs = render(new MasterChefRollMessage(), report);

		assertTrue(runs.stream().anyMatch(r -> " steal  no re-rolls from ".equals(r.text)));
		assertTrue(runs.stream().anyMatch(r -> "away".equals(r.text)));
	}

	@Test
	public void oneStolenUsesSingular() {
		given(game.getTeamHome().getId()).willReturn("home");
		given(game.getTeamAway().getId()).willReturn("away");
		given(game.getTeamHome().getName()).willReturn("home");

		ReportMasterChefRoll report = new ReportMasterChefRoll("away", new int[]{4, 5, 3}, 1);
		List<Run> runs = render(new MasterChefRollMessage(), report);

		assertTrue(runs.stream().anyMatch(r -> " steal 1 re-roll from ".equals(r.text)));
		assertTrue(runs.stream().anyMatch(r -> "home".equals(r.text)));
	}

	@Test
	public void multipleStolenUsesPlural() {
		given(game.getTeamHome().getId()).willReturn("home");
		given(game.getTeamAway().getId()).willReturn("away");

		ReportMasterChefRoll report = new ReportMasterChefRoll("home", new int[]{6, 6, 6}, 3);
		List<Run> runs = render(new MasterChefRollMessage(), report);

		assertTrue(runs.stream().anyMatch(r -> " steal 3 re-rolls from ".equals(r.text)));
	}

	@Test
	public void rollHeaderShowsAllThreeDice() {
		given(game.getTeamHome().getId()).willReturn("home");
		given(game.getTeamAway().getId()).willReturn("away");

		ReportMasterChefRoll report = new ReportMasterChefRoll("home", new int[]{1, 2, 3}, 0);
		List<Run> runs = render(new MasterChefRollMessage(), report);

		assertTrue(runs.stream().anyMatch(r -> "Master Chef Roll [ 1 ][ 2 ][ 3 ]".equals(r.text)
			&& r.textStyle == TextStyle.ROLL));
	}
}
