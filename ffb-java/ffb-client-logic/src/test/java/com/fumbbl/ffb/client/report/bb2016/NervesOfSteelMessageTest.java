package com.fumbbl.ffb.client.report.bb2016;

import com.fumbbl.ffb.client.report.ReportMessageTestBase;
import com.fumbbl.ffb.model.Player;
import com.fumbbl.ffb.report.bb2016.ReportNervesOfSteel;
import org.junit.jupiter.api.Test;
import org.mockito.Mock;

import java.util.List;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.BDDMockito.given;

class NervesOfSteelMessageTest extends ReportMessageTestBase {

	@SuppressWarnings("rawtypes")
	@Mock
	private Player player;

	@Test
	public void getKeyIsNervesOfSteel() {
		assertEquals("nervesOfSteel", new NervesOfSteelMessage().getKey());
	}

	@Test
	public void knownPlayerReportsBallAction() {
		given(game.getPlayerById("p1")).willReturn(player);

		ReportNervesOfSteel report = new ReportNervesOfSteel("p1", "pass");
		List<Run> runs = render(new NervesOfSteelMessage(), report);

		assertTrue(runs.stream().anyMatch(r -> " is using Nerves of Steel to pass the ball.".equals(r.text)));
	}

	@Test
	public void unknownPlayerProducesNoOutput() {
		given(game.getPlayerById("missing")).willReturn(null);

		ReportNervesOfSteel report = new ReportNervesOfSteel("missing", "catch");
		List<Run> runs = render(new NervesOfSteelMessage(), report);

		assertTrue(runs.isEmpty());
	}

	@Test
	public void catchActionText() {
		given(game.getPlayerById("p1")).willReturn(player);

		ReportNervesOfSteel report = new ReportNervesOfSteel("p1", "catch");
		List<Run> runs = render(new NervesOfSteelMessage(), report);

		assertTrue(runs.stream().anyMatch(r -> " is using Nerves of Steel to catch the ball.".equals(r.text)));
	}
}
