package com.fumbbl.ffb.client.report.mixed;

import com.fumbbl.ffb.client.report.ReportMessageTestBase;
import com.fumbbl.ffb.model.Player;
import com.fumbbl.ffb.report.mixed.ReportKickoffPitchInvasion;
import org.junit.jupiter.api.Test;
import org.mockito.Mock;

import java.util.Collections;
import java.util.List;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.mockito.BDDMockito.given;

class KickoffPitchInvasionMessageTest extends ReportMessageTestBase {

	@SuppressWarnings("rawtypes")
	@Mock
	private Player player;

	@Test
	public void rendersPitchInvasionRollWhenAmountPositivePlural() {
		given(game.getGameResult().getTeamResultHome().getFanFactor()).willReturn(3);
		given(game.getGameResult().getTeamResultAway().getFanFactor()).willReturn(2);

		ReportKickoffPitchInvasion report = new ReportKickoffPitchInvasion(3, 4, Collections.emptyList(), 2);
		List<Run> runs = render(new KickoffPitchInvasionMessage(), report);

		assertEquals("Pitch Invasion Roll [ 2 ]", runs.get(0).text);
		assertEquals("Affected Teams will have 2 players stunned.", runs.get(2).text);
		assertEquals("Pitch Invasion Roll Home Team [ 3 ]", runs.get(4).text);
		assertEquals("Rolled 3 + 3 Fan Factor = 6.", runs.get(6).text);
		assertEquals("Pitch Invasion Roll Away Team [ 4 ]", runs.get(8).text);
		assertEquals("Rolled 4 + 2 Fan Factor = 6.", runs.get(10).text);
	}

	@Test
	public void skipsAmountLinesWhenAmountZero() {
		given(game.getGameResult().getTeamResultHome().getFanFactor()).willReturn(0);
		given(game.getGameResult().getTeamResultAway().getFanFactor()).willReturn(0);

		ReportKickoffPitchInvasion report = new ReportKickoffPitchInvasion(1, 2, Collections.emptyList(), 0);
		List<Run> runs = render(new KickoffPitchInvasionMessage(), report);

		assertEquals("Pitch Invasion Roll Home Team [ 1 ]", runs.get(0).text);
	}

	@Test
	public void rendersSingularPlayerStunnedTextForAmountOne() {
		given(game.getGameResult().getTeamResultHome().getFanFactor()).willReturn(0);
		given(game.getGameResult().getTeamResultAway().getFanFactor()).willReturn(0);

		ReportKickoffPitchInvasion report = new ReportKickoffPitchInvasion(1, 2, Collections.emptyList(), 1);
		List<Run> runs = render(new KickoffPitchInvasionMessage(), report);

		assertEquals("Affected Teams will have 1 player stunned.", runs.get(2).text);
	}

	@Test
	public void rendersAffectedPlayers() {
		given(game.getGameResult().getTeamResultHome().getFanFactor()).willReturn(0);
		given(game.getGameResult().getTeamResultAway().getFanFactor()).willReturn(0);
		given(game.getPlayerById("p1")).willReturn(player);
		given(player.getName()).willReturn("Stunned Guy");

		ReportKickoffPitchInvasion report = new ReportKickoffPitchInvasion(1, 2, Collections.singletonList("p1"), 0);
		List<Run> runs = render(new KickoffPitchInvasionMessage(), report);

		int len = runs.size();
		assertEquals("Stunned Guy", runs.get(len - 3).text);
		assertEquals(" is stunned", runs.get(len - 2).text);
	}
}
