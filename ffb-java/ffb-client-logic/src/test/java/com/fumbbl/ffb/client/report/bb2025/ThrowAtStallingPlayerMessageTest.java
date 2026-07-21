package com.fumbbl.ffb.client.report.bb2025;

import com.fumbbl.ffb.client.report.ReportMessageTestBase;
import com.fumbbl.ffb.model.Player;
import com.fumbbl.ffb.report.mixed.ReportThrowAtStallingPlayer;
import org.junit.jupiter.api.Test;
import org.mockito.Mock;

import java.util.List;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.BDDMockito.given;

class ThrowAtStallingPlayerMessageTest extends ReportMessageTestBase {

	@SuppressWarnings("rawtypes")
	@Mock
	private Player staller;

	@Test
	public void getKeyIsThrowAtStallingPlayer() {
		assertEquals("throwAtStallingPlayer", new ThrowAtStallingPlayerMessage().getKey());
	}

	@Test
	public void noRollReportsCrowdNotBothered() {
		given(game.getPlayerById("p1")).willReturn(staller);
		given(staller.getName()).willReturn("Staller");
		given(game.getTeamHome().hasPlayer(staller)).willReturn(true);

		ReportThrowAtStallingPlayer report = new ReportThrowAtStallingPlayer("p1", 0, false);
		List<Run> runs = render(new ThrowAtStallingPlayerMessage(), report);

		assertTrue(runs.stream().anyMatch(r -> " stalled but the crowd can not be bothered.".equals(r.text)));
		assertFalse(runs.stream().anyMatch(r -> r.text != null && r.text.contains("Throw a Rock Roll")));
	}

	@Test
	public void successfulRollHitsWithRock() {
		given(game.getPlayerById("p1")).willReturn(staller);
		given(staller.getName()).willReturn("Staller");
		given(game.getTeamHome().hasPlayer(staller)).willReturn(true);

		ReportThrowAtStallingPlayer report = new ReportThrowAtStallingPlayer("p1", 5, true);
		List<Run> runs = render(new ThrowAtStallingPlayerMessage(), report);

		assertTrue(runs.stream().anyMatch(r -> "Throw a Rock Roll [ 5 ]".equals(r.text)));
		assertTrue(runs.stream().anyMatch(r -> " is hit by a rock.".equals(r.text)));
	}

	@Test
	public void unsuccessfulRollNotPunished() {
		given(game.getPlayerById("p1")).willReturn(staller);
		given(staller.getName()).willReturn("Staller");
		given(game.getTeamHome().hasPlayer(staller)).willReturn(true);

		ReportThrowAtStallingPlayer report = new ReportThrowAtStallingPlayer("p1", 1, false);
		List<Run> runs = render(new ThrowAtStallingPlayerMessage(), report);

		assertTrue(runs.stream().anyMatch(r -> " is not punished for stalling.".equals(r.text)));
	}
}
