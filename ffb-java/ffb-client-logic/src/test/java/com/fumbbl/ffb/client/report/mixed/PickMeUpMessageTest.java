package com.fumbbl.ffb.client.report.mixed;

import com.fumbbl.ffb.client.report.ReportMessageTestBase;
import com.fumbbl.ffb.model.Player;
import com.fumbbl.ffb.report.mixed.ReportPickMeUp;
import org.junit.jupiter.api.Test;
import org.mockito.Mock;

import java.util.List;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.mockito.BDDMockito.given;

class PickMeUpMessageTest extends ReportMessageTestBase {

	@SuppressWarnings("rawtypes")
	@Mock
	private Player player;

	@Test
	public void success() {
		given(game.getPlayerById("p1")).willReturn(player);
		given(player.getName()).willReturn("Grobnik");
		given(game.getTeamHome().hasPlayer(player)).willReturn(true);

		ReportPickMeUp report = new ReportPickMeUp("p1", 5, true);
		List<Run> runs = render(new PickMeUpMessage(), report);

		assertEquals("Pick-me-up Roll [ 5 ]", runs.get(0).text);
		assertEquals("Grobnik", runs.get(2).text);
		assertEquals(" is picked up.", runs.get(3).text);
	}

	@Test
	public void failure() {
		given(game.getPlayerById("p1")).willReturn(player);
		given(player.getName()).willReturn("Grobnik");
		given(game.getTeamHome().hasPlayer(player)).willReturn(true);

		ReportPickMeUp report = new ReportPickMeUp("p1", 1, false);
		List<Run> runs = render(new PickMeUpMessage(), report);

		assertEquals(" is not picked up.", runs.get(3).text);
	}

	@Test
	public void noPlayerId() {
		given(game.getPlayerById((String) null)).willReturn(null);

		ReportPickMeUp report = new ReportPickMeUp(null, 2, true);
		List<Run> runs = render(new PickMeUpMessage(), report);

		// No player-name run emitted; only roll header + result sentence (2 runs each).
		assertEquals(4, runs.size());
	}
}
