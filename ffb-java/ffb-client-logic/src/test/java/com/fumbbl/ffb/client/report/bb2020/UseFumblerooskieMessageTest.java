package com.fumbbl.ffb.client.report.bb2020;

import com.fumbbl.ffb.client.report.ReportMessageTestBase;
import com.fumbbl.ffb.model.Player;
import com.fumbbl.ffb.report.mixed.ReportFumblerooskie;
import org.junit.jupiter.api.Test;
import org.mockito.Mock;

import java.util.List;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.BDDMockito.given;

class UseFumblerooskieMessageTest extends ReportMessageTestBase {

	@SuppressWarnings("rawtypes")
	@Mock
	private Player player;

	@Test
	public void usedReportsWillDropBall() {
		given(game.getPlayerById("p1")).willReturn(player);

		ReportFumblerooskie report = new ReportFumblerooskie("p1", true);
		List<Run> runs = render(new UseFumblerooskieMessage(), report);
		assertTrue(runs.stream().anyMatch(r -> r.text != null && r.text.contains("will drop the ball using Fumblerooskie")));
	}

	@Test
	public void notUsedReportsKeepsBall() {
		given(game.getPlayerById("p1")).willReturn(player);

		ReportFumblerooskie report = new ReportFumblerooskie("p1", false);
		List<Run> runs = render(new UseFumblerooskieMessage(), report);
		assertTrue(runs.stream().anyMatch(r -> r.text != null && r.text.contains("did not vacate the square and thus keeps the ball.")));
	}

	@Test
	public void printsPlayerNameBeforeStatus() {
		given(game.getPlayerById("p1")).willReturn(player);
		given(player.getName()).willReturn("Player p1");

		ReportFumblerooskie report = new ReportFumblerooskie("p1", true);
		List<Run> runs = render(new UseFumblerooskieMessage(), report);
		assertEquals("Player p1", runs.get(0).text);
	}
}
