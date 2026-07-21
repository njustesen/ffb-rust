package com.fumbbl.ffb.client.report.bb2020;

import com.fumbbl.ffb.client.report.ReportMessageTestBase;
import com.fumbbl.ffb.model.Player;
import com.fumbbl.ffb.report.mixed.ReportThrowAtStallingPlayer;
import org.junit.jupiter.api.Test;
import org.mockito.Mock;

import java.util.List;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.BDDMockito.given;

class ThrowAtStallingPlayerMessageTest extends ReportMessageTestBase {

	@SuppressWarnings("rawtypes")
	@Mock
	private Player player;

	@Test
	public void successfulReportsHitByRock() {
		given(game.getPlayerById("p1")).willReturn(player);

		ReportThrowAtStallingPlayer report = new ReportThrowAtStallingPlayer("p1", 6, true);
		List<Run> runs = render(new ThrowAtStallingPlayerMessage(), report);
		assertTrue(runs.stream().anyMatch(r -> " is hit by a rock.".equals(r.text)));
	}

	@Test
	public void unsuccessfulReportsNotPunished() {
		given(game.getPlayerById("p1")).willReturn(player);

		ReportThrowAtStallingPlayer report = new ReportThrowAtStallingPlayer("p1", 1, false);
		List<Run> runs = render(new ThrowAtStallingPlayerMessage(), report);
		assertTrue(runs.stream().anyMatch(r -> " is not punished for stalling.".equals(r.text)));
	}

	@Test
	public void rollLineUsesCurrentIndent() {
		given(game.getPlayerById("p1")).willReturn(player);

		ReportThrowAtStallingPlayer report = new ReportThrowAtStallingPlayer("p1", 6, true);
		List<Run> runs = render(new ThrowAtStallingPlayerMessage(), report);
		assertEquals("Throw a Rock Roll [ 6 ]", runs.get(0).text);
	}
}
