package com.fumbbl.ffb.client.report.bb2020;

import com.fumbbl.ffb.client.report.ReportMessageTestBase;
import com.fumbbl.ffb.model.Player;
import com.fumbbl.ffb.report.bb2020.ReportKickoffOfficiousRef;
import org.junit.jupiter.api.Test;
import org.mockito.Mock;

import java.util.Collections;
import java.util.List;

import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.BDDMockito.given;

class KickoffOfficiousRefMessageTest extends ReportMessageTestBase {

	@SuppressWarnings("rawtypes")
	@Mock
	private Player player;

	@Test
	public void rendersRollsForBothTeams() {
		ReportKickoffOfficiousRef report = new ReportKickoffOfficiousRef(3, 5, Collections.emptyList());
		List<Run> runs = render(new KickoffOfficiousRefMessage(), report);
		assertTrue(runs.stream().anyMatch(r -> "Officious Ref Roll Home Team [ 3 ]".equals(r.text)));
		assertTrue(runs.stream().anyMatch(r -> "Officious Ref Roll Away Team [ 5 ]".equals(r.text)));
	}

	@Test
	public void noPlayersMeansNoArgumentLines() {
		ReportKickoffOfficiousRef report = new ReportKickoffOfficiousRef(1, 1, Collections.emptyList());
		List<Run> runs = render(new KickoffOfficiousRefMessage(), report);
		assertFalse(runs.stream().anyMatch(r -> " gets into an argument with the ref.".equals(r.text)));
	}

	@Test
	public void playerArgumentLineRendered() {
		given(game.getPlayerById("p1")).willReturn(player);
		given(player.getName()).willReturn("Bob");
		given(game.getTeamHome().hasPlayer(player)).willReturn(true);
		ReportKickoffOfficiousRef report = new ReportKickoffOfficiousRef(1, 1, Collections.singletonList("p1"));
		List<Run> runs = render(new KickoffOfficiousRefMessage(), report);
		assertTrue(runs.stream().anyMatch(r -> " gets into an argument with the ref.".equals(r.text)));
		assertTrue(runs.stream().anyMatch(r -> "Bob".equals(r.text)));
	}
}
