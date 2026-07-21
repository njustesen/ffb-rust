package com.fumbbl.ffb.client.report;

import com.fumbbl.ffb.client.TextStyle;
import com.fumbbl.ffb.model.Player;
import com.fumbbl.ffb.report.ReportRegenerationRoll;
import org.junit.jupiter.api.Test;
import org.mockito.Mock;

import java.util.List;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.BDDMockito.given;

class RegenerationRollMessageTest extends ReportMessageTestBase {

	@SuppressWarnings("rawtypes")
	@Mock
	private Player player;

	@Test
	public void renderSuccessfulRegeneration() {
		given(game.getPlayerById("p1")).willReturn(player);
		given(player.getName()).willReturn("Zombie");
		given(game.getTeamHome().hasPlayer(player)).willReturn(true);

		ReportRegenerationRoll report = new ReportRegenerationRoll("p1", true, 5, 4, false, null);
		List<Run> runs = render(new RegenerationRollMessage(), report);

		assertEquals("Regeneration Roll [ 5 ]", runs.get(0).text);
		assertEquals(TextStyle.ROLL, runs.get(0).textStyle);
		assertEquals("Zombie", runs.get(2).text);
		assertEquals(" regenerates.", runs.get(3).text);
	}

	@Test
	public void renderFailedRegeneration() {
		given(game.getPlayerById("p1")).willReturn(player);
		given(game.getTeamHome().hasPlayer(player)).willReturn(true);

		ReportRegenerationRoll report = new ReportRegenerationRoll("p1", false, 2, 4, false, null);
		List<Run> runs = render(new RegenerationRollMessage(), report);

		assertEquals(" does not regenerate.", runs.get(3).text);
	}

	@Test
	public void renderSkipsWhenRollIsZero() {
		ReportRegenerationRoll report = new ReportRegenerationRoll("p1", false, 0, 4, false, null);
		List<Run> runs = render(new RegenerationRollMessage(), report);

		assertTrue(runs.isEmpty());
	}
}
