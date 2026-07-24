package com.fumbbl.ffb.client.report;

import com.fumbbl.ffb.PlayerGender;
import com.fumbbl.ffb.client.TextStyle;
import com.fumbbl.ffb.model.Player;
import com.fumbbl.ffb.report.ReportEscapeRoll;
import org.junit.jupiter.api.Test;
import org.mockito.Mock;

import java.util.List;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.BDDMockito.given;

class EscapeRollMessageTest extends ReportMessageTestBase {

	@SuppressWarnings("rawtypes")
	@Mock
	private Player player;

	@Test
	public void successfulEscapeReportsWriggleFree() {
		given(game.getPlayerById("p1")).willReturn(player);
		given(player.getName()).willReturn("p1");
		given(game.getTeamHome().hasPlayer(player)).willReturn(true);

		ReportEscapeRoll report = new ReportEscapeRoll("p1", true, 4, 2, false, null);
		List<Run> runs = render(new EscapeRollMessage(), report);

		assertEquals("Escape Roll [ 4 ]", runs.get(0).text);
		assertEquals(TextStyle.ROLL, runs.get(0).textStyle);
		assertTrue(runs.stream().anyMatch(r -> r.text != null && r.text.contains("manages to wriggle free")));
	}

	@Test
	public void unsuccessfulEscapeReportsEatenWithGender() {
		given(game.getPlayerById("p2")).willReturn(player);
		given(player.getName()).willReturn("p2");
		given(player.getPlayerGender()).willReturn(PlayerGender.MALE);
		given(game.getTeamHome().hasPlayer(player)).willReturn(false);

		ReportEscapeRoll report = new ReportEscapeRoll("p2", false, 1, 3, false, null);
		List<Run> runs = render(new EscapeRollMessage(), report);

		assertTrue(runs.stream().anyMatch(r -> r.text != null && r.text.contains("team-mate's stomach")));
	}

	@Test
	public void printsPlayerAtIndentPlusOne() {
		given(game.getPlayerById("p3")).willReturn(player);
		given(player.getName()).willReturn("p3");
		given(game.getTeamHome().hasPlayer(player)).willReturn(true);

		ReportEscapeRoll report = new ReportEscapeRoll("p3", true, 5, 2, false, null);
		List<Run> runs = render(new EscapeRollMessage(), report);

		// run 0 is the roll status line, run 1 is its println terminator, run 2 is the player name print
		assertEquals("p3", runs.get(2).text);
		assertEquals(TextStyle.HOME, runs.get(2).textStyle);
	}

}
