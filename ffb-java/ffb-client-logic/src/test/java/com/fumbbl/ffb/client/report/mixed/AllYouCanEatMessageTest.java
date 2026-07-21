package com.fumbbl.ffb.client.report.mixed;

import com.fumbbl.ffb.PlayerGender;
import com.fumbbl.ffb.client.report.ReportMessageTestBase;
import com.fumbbl.ffb.model.Player;
import com.fumbbl.ffb.report.mixed.ReportAllYouCanEatRoll;
import org.junit.jupiter.api.Test;
import org.mockito.Mock;

import java.util.List;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.mockito.BDDMockito.given;

class AllYouCanEatMessageTest extends ReportMessageTestBase {

	@SuppressWarnings("rawtypes")
	@Mock
	private Player player;

	@Test
	public void successfulAndNotReRolledShowsGreetingAndSuccess() {
		given(game.getPlayerById("p1")).willReturn(player);
		given(player.getName()).willReturn("Grobnik");
		given(player.getPlayerGender()).willReturn(PlayerGender.MALE);
		given(game.getTeamHome().hasPlayer(player)).willReturn(true);

		ReportAllYouCanEatRoll report = new ReportAllYouCanEatRoll("p1", true, 4, 2, false);
		List<Run> runs = render(new AllYouCanEatMessage(), report);

		assertEquals("Grobnik", runs.get(0).text);
		assertEquals(" hopes the ref did not spot him.", runs.get(1).text);
		assertEquals("All You Can Eat Roll [ 4 ]", runs.get(3).text);
		assertEquals("Grobnik", runs.get(5).text);
		assertEquals(" goes unnoticed.", runs.get(6).text);
		assertEquals("Roll a 2+ to succeed", runs.get(8).text);
	}

	@Test
	public void unsuccessfulShowsSpotted() {
		given(game.getPlayerById("p1")).willReturn(player);
		given(player.getName()).willReturn("Grobnik");
		given(player.getPlayerGender()).willReturn(PlayerGender.MALE);
		given(game.getTeamHome().hasPlayer(player)).willReturn(true);

		ReportAllYouCanEatRoll report = new ReportAllYouCanEatRoll("p1", false, 1, 2, false);
		List<Run> runs = render(new AllYouCanEatMessage(), report);

		assertEquals(" is spotted.", runs.get(6).text);
	}

	@Test
	public void reRolledSkipsGreetingAndNeededRoll() {
		given(game.getPlayerById("p1")).willReturn(player);
		given(player.getName()).willReturn("Grobnik");
		given(game.getTeamHome().hasPlayer(player)).willReturn(true);

		ReportAllYouCanEatRoll report = new ReportAllYouCanEatRoll("p1", true, 4, 2, true);
		List<Run> runs = render(new AllYouCanEatMessage(), report);

		assertEquals(5, runs.size());
	}
}
