package com.fumbbl.ffb.client.report.bb2016;

import com.fumbbl.ffb.PlayerGender;
import com.fumbbl.ffb.client.report.ReportMessageTestBase;
import com.fumbbl.ffb.model.Player;
import com.fumbbl.ffb.report.bb2016.ReportArgueTheCallRoll;
import org.junit.jupiter.api.Test;
import org.mockito.Mock;

import java.util.List;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.BDDMockito.given;

class ArgueTheCallMessageTest extends ReportMessageTestBase {

	@SuppressWarnings("rawtypes")
	@Mock
	private Player player;

	@Test
	public void getKeyIsArgueTheCall() {
		assertEquals("argueTheCall", new ArgueTheCallMessage().getKey());
	}

	@Test
	public void successfulRollReportsRefRefrains() {
		given(game.getPlayerById("p1")).willReturn(player);
		given(player.getName()).willReturn("Grubb");
		given(player.getPlayerGender()).willReturn(PlayerGender.MALE);
		given(game.getTeamHome().hasPlayer(player)).willReturn(true);

		ReportArgueTheCallRoll report = new ReportArgueTheCallRoll("p1", true, false, 5);
		List<Run> runs = render(new ArgueTheCallMessage(), report);

		assertEquals("Argue the Call Roll [ 5 ]", runs.get(0).text);
		assertEquals("The ref refrains from banning ", runs.get(2).text);
	}

	@Test
	public void failedRollBansPlayer() {
		given(game.getPlayerById("p1")).willReturn(player);
		given(player.getName()).willReturn("Grubb");

		ReportArgueTheCallRoll report = new ReportArgueTheCallRoll("p1", false, false, 2);
		List<Run> runs = render(new ArgueTheCallMessage(), report);

		assertEquals("The ref bans ", runs.get(2).text);
	}

	@Test
	public void coachBannedReportsHomeCoachName() {
		given(game.getPlayerById("p1")).willReturn(player);
		given(player.getName()).willReturn("Grubb");
		given(game.getTeamHome().hasPlayer(player)).willReturn(true);
		given(game.getTeamHome().getCoach()).willReturn("Coachhome");

		ReportArgueTheCallRoll report = new ReportArgueTheCallRoll("p1", false, true, 2);
		List<Run> runs = render(new ArgueTheCallMessage(), report);

		assertTrue(runs.stream().anyMatch(r -> "Coachhome".equals(r.text)));
	}
}
