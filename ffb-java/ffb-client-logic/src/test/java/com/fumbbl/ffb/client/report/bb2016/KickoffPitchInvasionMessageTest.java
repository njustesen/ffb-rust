package com.fumbbl.ffb.client.report.bb2016;

import com.fumbbl.ffb.client.report.ReportMessageTestBase;
import com.fumbbl.ffb.model.Player;
import com.fumbbl.ffb.report.bb2016.ReportKickoffPitchInvasion;
import org.junit.jupiter.api.Test;
import org.mockito.Mock;

import java.util.List;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.BDDMockito.given;

class KickoffPitchInvasionMessageTest extends ReportMessageTestBase {

	@SuppressWarnings("rawtypes")
	@Mock
	private Player homePlayer;

	@SuppressWarnings("rawtypes")
	@Mock
	private Player awayPlayer;

	@Test
	public void getKeyIsKickoffPitchInvasion() {
		assertEquals("kickoffPitchInvasion", new KickoffPitchInvasionMessage().getKey());
	}

	@Test
	public void stunnedHomePlayerReportsRollAndEffect() {
		given(game.getTeamHome().getPlayers()).willReturn(new Player<?>[]{homePlayer});
		given(game.getTeamAway().getPlayers()).willReturn(new Player<?>[]{awayPlayer});

		ReportKickoffPitchInvasion report = new ReportKickoffPitchInvasion(new int[]{5}, new boolean[]{true}, new int[]{0}, new boolean[]{false});
		List<Run> runs = render(new KickoffPitchInvasionMessage(), report);

		assertEquals("Pitch Invasion Roll [ 5 ]", runs.get(0).text);
		assertTrue(runs.stream().anyMatch(r -> r.text != null && r.text.contains("has been stunned")));
	}

	@Test
	public void zeroRollPlayerProducesNoOutput() {
		given(game.getTeamHome().getPlayers()).willReturn(new Player<?>[]{homePlayer});
		given(game.getTeamAway().getPlayers()).willReturn(new Player<?>[]{awayPlayer});

		ReportKickoffPitchInvasion report = new ReportKickoffPitchInvasion(new int[]{0}, new boolean[]{false}, new int[]{0}, new boolean[]{false});
		List<Run> runs = render(new KickoffPitchInvasionMessage(), report);

		assertTrue(runs.isEmpty());
	}

	@Test
	public void unaffectedAwayPlayerReportsUnaffected() {
		given(game.getTeamHome().getPlayers()).willReturn(new Player<?>[]{homePlayer});
		given(game.getTeamAway().getPlayers()).willReturn(new Player<?>[]{awayPlayer});

		ReportKickoffPitchInvasion report = new ReportKickoffPitchInvasion(new int[]{0}, new boolean[]{false}, new int[]{3}, new boolean[]{false});
		List<Run> runs = render(new KickoffPitchInvasionMessage(), report);

		assertTrue(runs.stream().anyMatch(r -> r.text != null && r.text.contains("is unaffected")));
	}
}
