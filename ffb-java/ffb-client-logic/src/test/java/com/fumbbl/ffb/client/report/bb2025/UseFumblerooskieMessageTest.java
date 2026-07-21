package com.fumbbl.ffb.client.report.bb2025;

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
	private Player ballCarrier;

	@Test
	public void getKeyIsFumblerooskie() {
		assertEquals("fumblerooskie", new UseFumblerooskieMessage().getKey());
	}

	@Test
	public void usedReportsWillDropTheBall() {
		given(game.getPlayerById("p1")).willReturn(ballCarrier);
		given(ballCarrier.getName()).willReturn("Ball Carrier");
		given(game.getTeamHome().hasPlayer(ballCarrier)).willReturn(true);

		ReportFumblerooskie report = new ReportFumblerooskie("p1", true);
		List<Run> runs = render(new UseFumblerooskieMessage(), report);

		assertTrue(runs.stream().anyMatch(r -> r.text != null && r.text.contains("will drop the ball using Fumblerooski")));
	}

	@Test
	public void notUsedReportsKeepsTheBall() {
		given(game.getPlayerById("p1")).willReturn(ballCarrier);
		given(ballCarrier.getName()).willReturn("Ball Carrier");
		given(game.getTeamHome().hasPlayer(ballCarrier)).willReturn(true);

		ReportFumblerooskie report = new ReportFumblerooskie("p1", false);
		List<Run> runs = render(new UseFumblerooskieMessage(), report);

		assertTrue(runs.stream().anyMatch(r -> r.text != null && r.text.contains("did not vacate the square and thus keeps the ball.")));
	}

	@Test
	public void printsPlayerName() {
		given(game.getPlayerById("p1")).willReturn(ballCarrier);
		given(ballCarrier.getName()).willReturn("Ball Carrier");
		given(game.getTeamHome().hasPlayer(ballCarrier)).willReturn(true);

		ReportFumblerooskie report = new ReportFumblerooskie("p1", true);
		List<Run> runs = render(new UseFumblerooskieMessage(), report);

		assertTrue(runs.stream().anyMatch(r -> "Ball Carrier".equals(r.text)));
	}
}
