package com.fumbbl.ffb.client.report.mixed;

import com.fumbbl.ffb.client.ParagraphStyle;
import com.fumbbl.ffb.client.report.ReportMessageTestBase;
import com.fumbbl.ffb.model.Player;
import com.fumbbl.ffb.report.mixed.ReportPlayerEvent;
import org.junit.jupiter.api.Test;
import org.mockito.Mock;

import java.util.List;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.mockito.BDDMockito.given;

class PlayerEventMessageTest extends ReportMessageTestBase {

	@SuppressWarnings("rawtypes")
	@Mock
	private Player player;

	@Test
	public void rendersPlayerAndEventMessage() {
		given(game.getPlayerById("p1")).willReturn(player);
		given(player.getName()).willReturn("Grobnik");
		given(game.getTeamHome().hasPlayer(player)).willReturn(true);

		ReportPlayerEvent report = new ReportPlayerEvent("p1", "gets a niggling injury.");
		List<Run> runs = render(new PlayerEventMessage(), report);

		assertEquals("Grobnik", runs.get(0).text);
		assertEquals(" gets a niggling injury.", runs.get(1).text);
	}

	@Test
	public void indentIsOffsetByOne() {
		given(game.getPlayerById("p1")).willReturn(player);
		given(player.getName()).willReturn("Grobnik");
		given(game.getTeamHome().hasPlayer(player)).willReturn(true);
		statusReport.setIndent(1);

		ReportPlayerEvent report = new ReportPlayerEvent("p1", "event");
		List<Run> runs = render(new PlayerEventMessage(), report);

		assertEquals(ParagraphStyle.INDENT_2, runs.get(0).paragraphStyle);
	}

	@Test
	public void missingEventMessageRendersEmpty() {
		given(game.getPlayerById("p1")).willReturn(player);
		given(player.getName()).willReturn("Grobnik");
		given(game.getTeamHome().hasPlayer(player)).willReturn(true);

		ReportPlayerEvent report = new ReportPlayerEvent("p1", null);
		List<Run> runs = render(new PlayerEventMessage(), report);

		assertEquals(" null", runs.get(1).text);
	}
}
