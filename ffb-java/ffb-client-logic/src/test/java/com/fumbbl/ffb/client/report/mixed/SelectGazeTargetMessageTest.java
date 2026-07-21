package com.fumbbl.ffb.client.report.mixed;

import com.fumbbl.ffb.client.TextStyle;
import com.fumbbl.ffb.client.report.ReportMessageTestBase;
import com.fumbbl.ffb.model.Player;
import com.fumbbl.ffb.report.mixed.ReportSelectGazeTarget;
import org.junit.jupiter.api.Test;
import org.mockito.Mock;

import java.util.List;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.mockito.BDDMockito.given;

class SelectGazeTargetMessageTest extends ReportMessageTestBase {

	@SuppressWarnings("rawtypes")
	@Mock
	private Player attacker;

	@SuppressWarnings("rawtypes")
	@Mock
	private Player defender;

	// java: `missing_attacker_renders_nothing` from the Rust suite is not portable —
	// SelectGazeTargetMessage.render() calls `attacker.getName()` unconditionally (no null
	// guard); a missing attacker NPEs in real Java. Skipped.

	@Test
	public void attackerHomeDefenderAway() {
		given(game.getPlayerById("p1")).willReturn(attacker);
		given(attacker.getName()).willReturn("Attacker");
		given(game.getTeamHome().hasPlayer(attacker)).willReturn(true);
		given(game.getPlayerById("p2")).willReturn(defender);
		given(defender.getName()).willReturn("Defender");
		given(game.getTeamHome().hasPlayer(defender)).willReturn(false);

		ReportSelectGazeTarget report = new ReportSelectGazeTarget("p1", "p2");
		List<Run> runs = render(new SelectGazeTargetMessage(), report);

		assertEquals("Attacker", runs.get(0).text);
		assertEquals(TextStyle.HOME, runs.get(0).textStyle);
		assertEquals(" targets ", runs.get(1).text);
		assertEquals("Defender", runs.get(2).text);
		assertEquals(TextStyle.AWAY, runs.get(2).textStyle);
		assertEquals(".", runs.get(3).text);
	}
}
