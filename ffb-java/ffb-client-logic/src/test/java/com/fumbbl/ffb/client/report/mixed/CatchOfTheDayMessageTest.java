package com.fumbbl.ffb.client.report.mixed;

import com.fumbbl.ffb.client.report.ReportMessageTestBase;
import com.fumbbl.ffb.model.Player;
import com.fumbbl.ffb.report.mixed.ReportCatchOfTheDayRoll;
import org.junit.jupiter.api.Test;
import org.mockito.Mock;

import java.util.List;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.mockito.BDDMockito.given;

class CatchOfTheDayMessageTest extends ReportMessageTestBase {

	@SuppressWarnings("rawtypes")
	@Mock
	private Player player;

	@Test
	public void successWithoutReRollShowsIntroAndNeededRoll() {
		given(game.getPlayerById("p1")).willReturn(player);
		given(player.getName()).willReturn("Scavenger");
		given(game.getTeamHome().hasPlayer(player)).willReturn(true);

		ReportCatchOfTheDayRoll report = new ReportCatchOfTheDayRoll("p1", true, 4, 2, false);
		List<Run> runs = render(new CatchOfTheDayMessage(), report);

		assertEquals("Scavenger", runs.get(0).text);
		assertEquals(" tries to get the ball from the ground:", runs.get(1).text);
		assertEquals("Catch of the Day Roll [ 4 ]", runs.get(3).text);
		assertEquals("Scavenger", runs.get(5).text);
		assertEquals(" gets the ball.", runs.get(6).text);
		assertEquals("Succeeded on a roll of 2+", runs.get(8).text);
	}

	@Test
	public void failureShowsFailsToGetBall() {
		given(game.getPlayerById("p1")).willReturn(player);
		given(player.getName()).willReturn("Scavenger");
		given(game.getTeamHome().hasPlayer(player)).willReturn(true);

		ReportCatchOfTheDayRoll report = new ReportCatchOfTheDayRoll("p1", false, 1, 2, false);
		List<Run> runs = render(new CatchOfTheDayMessage(), report);

		assertEquals(" fails to get the ball.", runs.get(6).text);
		assertEquals("Roll a 2+ to succeed", runs.get(8).text);
	}

	@Test
	public void reRolledSkipsIntroAndNeededRoll() {
		given(game.getPlayerById("p1")).willReturn(player);
		given(player.getName()).willReturn("Scavenger");
		given(game.getTeamHome().hasPlayer(player)).willReturn(true);

		ReportCatchOfTheDayRoll report = new ReportCatchOfTheDayRoll("p1", true, 4, 2, true);
		List<Run> runs = render(new CatchOfTheDayMessage(), report);

		assertEquals(5, runs.size());
	}
}
