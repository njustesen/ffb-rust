package com.fumbbl.ffb.client.report.mixed;

import com.fumbbl.ffb.client.report.ReportMessageTestBase;
import com.fumbbl.ffb.model.Player;
import com.fumbbl.ffb.report.mixed.ReportNervesOfSteel;
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
	public void bombThrow() {
		given(game.getPlayerById("p1")).willReturn(player);
		given(player.getName()).willReturn("Grobnik");
		given(game.getTeamHome().hasPlayer(player)).willReturn(true);

		ReportNervesOfSteel report = new ReportNervesOfSteel("p1", true);
		List<Run> runs = render(new NervesOfSteelMessage(), report);

		assertEquals("Grobnik", runs.get(0).text);
		assertEquals("throw the bomb.", runs.get(2).text);
	}

	@Test
	public void ballActionUsedWhenNotBomb() {
		given(game.getPlayerById("p1")).willReturn(player);
		given(player.getName()).willReturn("Grobnik");
		given(game.getTeamHome().hasPlayer(player)).willReturn(true);

		ReportNervesOfSteel report = new ReportNervesOfSteel("p1", "pass");
		List<Run> runs = render(new NervesOfSteelMessage(), report);

		assertEquals("pass the ball.", runs.get(2).text);
	}

	@Test
	public void noPlayerRendersNothing() {
		given(game.getPlayerById("unknown")).willReturn(null);

		ReportNervesOfSteel report = new ReportNervesOfSteel("unknown", "pass");
		List<Run> runs = render(new NervesOfSteelMessage(), report);

		assertTrue(runs.isEmpty());
	}
}
