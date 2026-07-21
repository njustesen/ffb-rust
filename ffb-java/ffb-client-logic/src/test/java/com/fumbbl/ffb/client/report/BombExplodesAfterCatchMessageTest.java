package com.fumbbl.ffb.client.report;

import com.fumbbl.ffb.PlayerGender;
import com.fumbbl.ffb.client.ParagraphStyle;
import com.fumbbl.ffb.client.TextStyle;
import com.fumbbl.ffb.model.Player;
import com.fumbbl.ffb.report.ReportBombExplodesAfterCatch;
import org.junit.jupiter.api.Test;
import org.mockito.Mock;

import java.util.List;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.mockito.BDDMockito.given;

class BombExplodesAfterCatchMessageTest extends ReportMessageTestBase {

	@SuppressWarnings("rawtypes")
	@Mock
	private Player player;

	@Test
	public void explodesUsesGenitiveAndHomeStyle() {
		given(game.getPlayerById("p1")).willReturn(player);
		given(player.getName()).willReturn("Catcher");
		given(player.getPlayerGender()).willReturn(PlayerGender.MALE);
		given(game.getTeamHome().hasPlayer(player)).willReturn(true);

		ReportBombExplodesAfterCatch report = new ReportBombExplodesAfterCatch("p1", true, 5);
		List<Run> runs = render(new BombExplodesAfterCatchMessage(), report);

		assertEquals("Bomb Roll [ 5 ]", runs.get(0).text);
		assertEquals("Catcher", runs.get(2).text);
		assertEquals(TextStyle.HOME, runs.get(2).textStyle);
		assertEquals(" caught the bomb", runs.get(3).text);
		assertEquals(" but it explodes in his hands.", runs.get(4).text);
	}

	@Test
	public void doesNotExplodeAwayStyle() {
		given(game.getPlayerById("p2")).willReturn(player);
		given(player.getName()).willReturn("AwayCatcher");
		given(game.getTeamHome().hasPlayer(player)).willReturn(false);

		ReportBombExplodesAfterCatch report = new ReportBombExplodesAfterCatch("p2", false, 2);
		List<Run> runs = render(new BombExplodesAfterCatchMessage(), report);

		assertEquals(TextStyle.AWAY, runs.get(2).textStyle);
		assertEquals(" and it does not explode", runs.get(4).text);
	}

	@Test
	public void honorsCurrentIndent() {
		given(game.getPlayerById("p1")).willReturn(player);
		given(player.getName()).willReturn("Catcher");
		given(game.getTeamHome().hasPlayer(player)).willReturn(true);

		ReportBombExplodesAfterCatch report = new ReportBombExplodesAfterCatch("p1", false, 1);
		statusReport.setIndent(1);
		List<Run> runs = render(new BombExplodesAfterCatchMessage(), report);

		assertEquals(ParagraphStyle.INDENT_2, runs.get(0).paragraphStyle);
	}
}
