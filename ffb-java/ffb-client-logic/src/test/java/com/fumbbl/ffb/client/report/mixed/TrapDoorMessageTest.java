package com.fumbbl.ffb.client.report.mixed;

import com.fumbbl.ffb.client.report.ReportMessageTestBase;
import com.fumbbl.ffb.model.Player;
import com.fumbbl.ffb.report.mixed.ReportTrapDoor;
import org.junit.jupiter.api.Test;
import org.mockito.Mock;

import java.util.List;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.mockito.BDDMockito.given;

class TrapDoorMessageTest extends ReportMessageTestBase {

	@SuppressWarnings("rawtypes")
	@Mock
	private Player player;

	@Test
	public void escapedTrapdoor() {
		given(game.getPlayerById("p1")).willReturn(player);
		given(player.getName()).willReturn("Faller");
		given(game.getTeamHome().hasPlayer(player)).willReturn(true);

		ReportTrapDoor report = new ReportTrapDoor("p1", 4, true);
		List<Run> runs = render(new TrapDoorMessage(), report);

		// run0 = roll text, run1 = println terminator, run2 = player name, run3 = outcome text.
		assertEquals("Trapdoor Roll [ 4 ]", runs.get(0).text);
		assertEquals("Faller", runs.get(2).text);
		assertEquals(" escapes the trapdoor.", runs.get(3).text);
	}

	@Test
	public void fallsDownTrapdoor() {
		given(game.getPlayerById("p1")).willReturn(player);
		given(player.getName()).willReturn("Faller");
		given(game.getTeamHome().hasPlayer(player)).willReturn(true);

		ReportTrapDoor report = new ReportTrapDoor("p1", 2, false);
		List<Run> runs = render(new TrapDoorMessage(), report);

		assertEquals(" falls down the trapdoor.", runs.get(3).text);
	}

	@Test
	public void missingPlayerStillReportsRollAndOutcome() {
		// without this stub, the deep-stub Game mock would auto-vivify a non-null Player
		// mock for the unstubbed getPlayerById(null) call, so the print(indent, false, player)
		// no-op branch would never trigger.
		given(game.getPlayerById((String) null)).willReturn(null);

		ReportTrapDoor report = new ReportTrapDoor(null, 6, true);
		List<Run> runs = render(new TrapDoorMessage(), report);

		// no player run since print is a no-op for null: roll text + terminator + outcome text + terminator.
		assertEquals(4, runs.size());
		assertEquals(" escapes the trapdoor.", runs.get(2).text);
	}
}
