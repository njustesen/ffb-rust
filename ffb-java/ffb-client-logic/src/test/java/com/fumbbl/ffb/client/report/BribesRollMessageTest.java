package com.fumbbl.ffb.client.report;

import com.fumbbl.ffb.PlayerGender;
import com.fumbbl.ffb.model.Player;
import com.fumbbl.ffb.report.ReportBribesRoll;
import org.junit.jupiter.api.Test;
import org.mockito.Mock;

import java.util.List;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.mockito.BDDMockito.given;

class BribesRollMessageTest extends ReportMessageTestBase {

	@SuppressWarnings("rawtypes")
	@Mock
	private Player player;

	@Test
	public void successfulSingularGenderRemains() {
		given(game.getPlayerById("p1")).willReturn(player);
		given(player.getName()).willReturn("Bribed");
		given(player.getPlayerGender()).willReturn(PlayerGender.MALE);
		given(game.getTeamHome().hasPlayer(player)).willReturn(true);

		ReportBribesRoll report = new ReportBribesRoll("p1", true, 5);
		List<Run> runs = render(new BribesRollMessage(), report);

		assertEquals("Bribes Roll [ 5 ]", runs.get(0).text);
		assertEquals("The ref refrains from penalizing ", runs.get(2).text);
		assertEquals("Bribed", runs.get(3).text);
		assertEquals(" and he remains in the game.", runs.get(4).text);
	}

	@Test
	public void successfulNonbinaryGenderRemainPlural() {
		given(game.getPlayerById("p1")).willReturn(player);
		given(player.getName()).willReturn("Bribed");
		given(player.getPlayerGender()).willReturn(PlayerGender.NONBINARY);
		given(game.getTeamHome().hasPlayer(player)).willReturn(true);

		ReportBribesRoll report = new ReportBribesRoll("p1", true, 5);
		List<Run> runs = render(new BribesRollMessage(), report);

		assertEquals(" and they remain in the game.", runs.get(4).text);
	}

	@Test
	public void unsuccessfulPlayerMustLeave() {
		given(game.getPlayerById("p2")).willReturn(player);
		given(player.getName()).willReturn("Penalized");
		given(game.getTeamHome().hasPlayer(player)).willReturn(false);

		ReportBribesRoll report = new ReportBribesRoll("p2", false, 1);
		List<Run> runs = render(new BribesRollMessage(), report);

		assertEquals("The ref appears to be unimpressed and ", runs.get(2).text);
		assertEquals("Penalized", runs.get(3).text);
		assertEquals(" must leave the game.", runs.get(4).text);
	}
}
