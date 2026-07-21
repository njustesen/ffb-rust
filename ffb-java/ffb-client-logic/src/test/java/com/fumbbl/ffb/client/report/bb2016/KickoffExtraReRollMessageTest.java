package com.fumbbl.ffb.client.report.bb2016;

import com.fumbbl.ffb.client.report.ReportMessageTestBase;
import com.fumbbl.ffb.kickoff.bb2016.KickoffResult;
import com.fumbbl.ffb.model.Player;
import com.fumbbl.ffb.report.bb2016.ReportKickoffExtraReRoll;
import org.junit.jupiter.api.Test;

import java.util.List;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.BDDMockito.given;

class KickoffExtraReRollMessageTest extends ReportMessageTestBase {

	@Test
	public void getKeyIsKickoffExtraReRoll() {
		assertEquals("extraReRoll", new KickoffExtraReRollMessage().getKey());
	}

	@Test
	public void homeGainsReRollReportsTeamName() {
		given(game.getTeamHome().getPlayers()).willReturn(new Player<?>[0]);
		given(game.getTeamAway().getPlayers()).willReturn(new Player<?>[0]);

		ReportKickoffExtraReRoll report = new ReportKickoffExtraReRoll(KickoffResult.WEATHER_CHANGE, 0, true, 0, false);
		List<Run> runs = render(new KickoffExtraReRollMessage(), report);

		assertTrue(runs.stream().anyMatch(r -> " gains a Re-Roll.".equals(r.text)));
	}

	@Test
	public void fanRerollKickoffReportsCheeringFansRolls() {
		given(game.getTeamHome().getPlayers()).willReturn(new Player<?>[0]);
		given(game.getTeamAway().getPlayers()).willReturn(new Player<?>[0]);

		ReportKickoffExtraReRoll report = new ReportKickoffExtraReRoll(KickoffResult.CHEERING_FANS, 4, false, 5, false);
		List<Run> runs = render(new KickoffExtraReRollMessage(), report);

		assertEquals("Cheering Fans Roll Home Team [ 4 ]", runs.get(0).text);
	}

	@Test
	public void coachRerollKickoffReportsBrilliantCoachingRolls() {
		given(game.getTeamHome().getPlayers()).willReturn(new Player<?>[0]);
		given(game.getTeamAway().getPlayers()).willReturn(new Player<?>[0]);

		ReportKickoffExtraReRoll report = new ReportKickoffExtraReRoll(KickoffResult.BRILLIANT_COACHING, 3, false, 2, false);
		List<Run> runs = render(new KickoffExtraReRollMessage(), report);

		assertTrue(runs.stream().anyMatch(r -> "Brilliant Coaching Roll Home Team [ 3 ]".equals(r.text)));
	}
}
