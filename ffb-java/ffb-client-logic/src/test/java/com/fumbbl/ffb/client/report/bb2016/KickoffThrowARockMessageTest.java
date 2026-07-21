package com.fumbbl.ffb.client.report.bb2016;

import com.fumbbl.ffb.client.report.ReportMessageTestBase;
import com.fumbbl.ffb.model.Player;
import com.fumbbl.ffb.report.bb2016.ReportKickoffThrowARock;
import org.junit.jupiter.api.Test;
import org.mockito.Mock;

import java.util.List;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.BDDMockito.given;

class KickoffThrowARockMessageTest extends ReportMessageTestBase {

	@SuppressWarnings("rawtypes")
	@Mock
	private Player grubb;

	@Test
	public void getKeyIsKickoffThrowARock() {
		assertEquals("kickoffThrowARock", new KickoffThrowARockMessage().getKey());
	}

	@Test
	public void reportsBothTeamRolls() {
		given(game.getTeamHome().getPlayers()).willReturn(new Player<?>[0]);
		given(game.getTeamAway().getPlayers()).willReturn(new Player<?>[0]);

		ReportKickoffThrowARock report = new ReportKickoffThrowARock(3, 5, new String[0]);
		List<Run> runs = render(new KickoffThrowARockMessage(), report);

		assertEquals("Throw a Rock Roll Home Team [ 3 ]", runs.get(0).text);
		assertTrue(runs.stream().anyMatch(r -> "Throw a Rock Roll Away Team [ 5 ]".equals(r.text)));
	}

	@Test
	public void reportsHitPlayers() {
		given(game.getTeamHome().getPlayers()).willReturn(new Player<?>[0]);
		given(game.getTeamAway().getPlayers()).willReturn(new Player<?>[0]);
		given(game.getPlayerById("p1")).willReturn(grubb);
		given(grubb.getName()).willReturn("Grubb");

		ReportKickoffThrowARock report = new ReportKickoffThrowARock(3, 5, new String[]{"p1"});
		List<Run> runs = render(new KickoffThrowARockMessage(), report);

		assertTrue(runs.stream().anyMatch(r -> "Grubb".equals(r.text)));
		assertTrue(runs.stream().anyMatch(r -> " is hit by a rock.".equals(r.text)));
	}

	@Test
	public void noHitPlayersProducesNoHitLines() {
		given(game.getTeamHome().getPlayers()).willReturn(new Player<?>[0]);
		given(game.getTeamAway().getPlayers()).willReturn(new Player<?>[0]);

		ReportKickoffThrowARock report = new ReportKickoffThrowARock(1, 1, new String[0]);
		List<Run> runs = render(new KickoffThrowARockMessage(), report);

		assertFalse(runs.stream().anyMatch(r -> " is hit by a rock.".equals(r.text)));
	}
}
