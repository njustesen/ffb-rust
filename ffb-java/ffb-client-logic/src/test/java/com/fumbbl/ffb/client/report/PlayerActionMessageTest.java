package com.fumbbl.ffb.client.report;

import com.fumbbl.ffb.PlayerAction;
import com.fumbbl.ffb.client.TextStyle;
import com.fumbbl.ffb.model.Player;
import com.fumbbl.ffb.report.ReportPlayerAction;
import org.junit.jupiter.api.Test;
import org.mockito.Mock;

import java.util.List;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.BDDMockito.given;

class PlayerActionMessageTest extends ReportMessageTestBase {

	@SuppressWarnings("rawtypes")
	@Mock
	private Player player;

	@Test
	public void renderPrintsPlayerAndDescriptionWhenPresent() {
		given(game.getPlayerById("p1")).willReturn(player);
		given(player.getName()).willReturn("Grombrindal");
		given(game.getTeamHome().hasPlayer(player)).willReturn(true);

		ReportPlayerAction report = new ReportPlayerAction("p1", PlayerAction.MOVE);
		List<Run> runs = render(new PlayerActionMessage(), report);

		assertEquals("Grombrindal", runs.get(0).text);
		assertEquals(" starts a Move Action.", runs.get(1).text);
		assertEquals(TextStyle.BOLD, runs.get(1).textStyle);
	}

	@Test
	public void renderSkipsOutputWhenDescriptionIsNone() {
		given(game.getPlayerById("p1")).willReturn(player);

		ReportPlayerAction report = new ReportPlayerAction("p1", PlayerAction.BLITZ);
		List<Run> runs = render(new PlayerActionMessage(), report);

		assertTrue(runs.isEmpty());
	}

	@Test
	public void renderSkipsOutputWhenPlayerMissing() {
		given(game.getPlayerById("nobody")).willReturn(null);

		ReportPlayerAction report = new ReportPlayerAction("nobody", PlayerAction.MOVE);
		List<Run> runs = render(new PlayerActionMessage(), report);

		assertTrue(runs.isEmpty());
	}

	@Test
	public void renderSetsIndentToOneAfterward() {
		given(game.getPlayerById("nobody")).willReturn(null);
		statusReport.setIndent(5);
		ReportPlayerAction report = new ReportPlayerAction("nobody", PlayerAction.MOVE);
		render(new PlayerActionMessage(), report);

		assertEquals(1, statusReport.getIndent());
	}
}
