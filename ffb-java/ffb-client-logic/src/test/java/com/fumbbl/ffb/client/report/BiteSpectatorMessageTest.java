package com.fumbbl.ffb.client.report;

import com.fumbbl.ffb.client.TextStyle;
import com.fumbbl.ffb.model.Player;
import com.fumbbl.ffb.report.ReportBiteSpectator;
import org.junit.jupiter.api.Test;
import org.mockito.Mock;

import java.util.List;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.BDDMockito.given;

class BiteSpectatorMessageTest extends ReportMessageTestBase {

	@SuppressWarnings("rawtypes")
	@Mock
	private Player player;

	@Test
	public void rendersForKnownPlayer() {
		given(game.getPlayerById("p1")).willReturn(player);
		given(player.getName()).willReturn("Biter");
		given(game.getTeamHome().hasPlayer(player)).willReturn(true);

		ReportBiteSpectator report = new ReportBiteSpectator("p1");
		List<Run> runs = render(new BiteSpectatorMessage(), report);

		assertEquals("Biter", runs.get(0).text);
		assertEquals(TextStyle.HOME_BOLD, runs.get(0).textStyle);
		assertEquals(" heads off to the spectator ranks to bite some beautiful maiden.", runs.get(1).text);
	}

	@Test
	public void rendersAwayPlayerBold() {
		given(game.getPlayerById("p2")).willReturn(player);
		given(player.getName()).willReturn("AwayBiter");
		given(game.getTeamHome().hasPlayer(player)).willReturn(false);

		ReportBiteSpectator report = new ReportBiteSpectator("p2");
		List<Run> runs = render(new BiteSpectatorMessage(), report);

		assertEquals(TextStyle.AWAY_BOLD, runs.get(0).textStyle);
	}

	@Test
	public void skipsRenderForUnknownPlayer() {
		given(game.getPlayerById("missing")).willReturn(null);

		ReportBiteSpectator report = new ReportBiteSpectator("missing");
		List<Run> runs = render(new BiteSpectatorMessage(), report);

		assertTrue(runs.isEmpty());
	}
}
