package com.fumbbl.ffb.client.report.mixed;

import com.fumbbl.ffb.PlayerGender;
import com.fumbbl.ffb.client.TextStyle;
import com.fumbbl.ffb.client.report.ReportMessageTestBase;
import com.fumbbl.ffb.model.Player;
import com.fumbbl.ffb.report.mixed.ReportArgueTheCallRoll;
import org.junit.jupiter.api.Test;
import org.mockito.Mock;

import java.util.List;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.mockito.BDDMockito.given;

class ArgueTheCallMessageTest extends ReportMessageTestBase {

	@SuppressWarnings("rawtypes")
	@Mock
	private Player player;

	@Test
	public void successfulReportsStaysOnPitch() {
		given(game.getPlayerById("p1")).willReturn(player);
		given(player.getName()).willReturn("Grobnik");
		given(player.getPlayerGender()).willReturn(PlayerGender.MALE);
		given(game.getTeamHome().hasPlayer(player)).willReturn(true);

		ReportArgueTheCallRoll report = new ReportArgueTheCallRoll("p1", true, false, 6, true, false, 0);
		List<Run> runs = render(new ArgueTheCallMessage(), report);

		assertEquals("Argue the Call Roll [ 6 ]", runs.get(0).text);
		assertEquals("The ref refrains from banning ", runs.get(2).text);
		assertEquals("Grobnik", runs.get(3).text);
		assertEquals(" and he stays on the pitch.", runs.get(4).text);
		assertEquals("Succeeded on a roll of 6 (Roll >= 6)", runs.get(6).text);
	}

	@Test
	public void successfulReportsSentToReserve() {
		given(game.getPlayerById("p1")).willReturn(player);
		given(player.getName()).willReturn("Grobnik");
		given(player.getPlayerGender()).willReturn(PlayerGender.MALE);
		given(game.getTeamHome().hasPlayer(player)).willReturn(true);

		ReportArgueTheCallRoll report = new ReportArgueTheCallRoll("p1", true, false, 6, false, false, 0);
		List<Run> runs = render(new ArgueTheCallMessage(), report);

		assertEquals(" and he is sent to the reserve instead.", runs.get(4).text);
	}

	@Test
	public void failureReportsBanned() {
		given(game.getPlayerById("p1")).willReturn(player);
		given(player.getName()).willReturn("Grobnik");
		given(player.getPlayerGender()).willReturn(PlayerGender.MALE);
		given(game.getTeamHome().hasPlayer(player)).willReturn(true);

		ReportArgueTheCallRoll report = new ReportArgueTheCallRoll("p1", false, false, 3, false, false, 0);
		List<Run> runs = render(new ArgueTheCallMessage(), report);

		assertEquals("The ref bans ", runs.get(2).text);
		assertEquals(" from the game.", runs.get(4).text);
		assertEquals("Would have succeeded on a roll of 6 (Roll >= 6)", runs.get(6).text);
	}

	@Test
	public void friendsWithRefLowersTargetAndAddsExplanation() {
		given(game.getPlayerById("p1")).willReturn(player);
		given(player.getName()).willReturn("Grobnik");
		given(player.getPlayerGender()).willReturn(PlayerGender.MALE);
		given(game.getTeamHome().hasPlayer(player)).willReturn(true);

		ReportArgueTheCallRoll report = new ReportArgueTheCallRoll("p1", true, false, 5, true, true, 0);
		List<Run> runs = render(new ArgueTheCallMessage(), report);

		assertEquals("Argue the Call Roll [ 5 ]", runs.get(0).text);
		assertEquals("Being friends with the ref allows argue to succeed on 5+.", runs.get(2).text);
	}

	@Test
	public void biasedRefsAddModifierAndLowerMinimum() {
		given(game.getPlayerById("p1")).willReturn(player);
		given(player.getName()).willReturn("Grobnik");
		given(player.getPlayerGender()).willReturn(PlayerGender.MALE);
		given(game.getTeamHome().hasPlayer(player)).willReturn(true);

		ReportArgueTheCallRoll report = new ReportArgueTheCallRoll("p1", true, false, 6, true, false, 2);
		List<Run> runs = render(new ArgueTheCallMessage(), report);

		assertEquals("Succeeded on a roll of 4 (Roll + 2 Biased Referees >= 6)", runs.get(6).text);
	}

	@Test
	public void coachBannedPrintsHomeCoach() {
		given(game.getPlayerById("p1")).willReturn(player);
		given(player.getName()).willReturn("Grobnik");
		given(player.getPlayerGender()).willReturn(PlayerGender.MALE);
		given(game.getTeamHome().hasPlayer(player)).willReturn(true);
		given(game.getTeamHome().getCoach()).willReturn("HomeCoach");

		ReportArgueTheCallRoll report = new ReportArgueTheCallRoll("p1", false, true, 2, false, false, 0);
		List<Run> runs = render(new ArgueTheCallMessage(), report);

		Run coachNameRun = runs.stream().filter(r -> "HomeCoach".equals(r.text)).findFirst().orElseThrow();
		assertEquals(TextStyle.HOME, coachNameRun.textStyle);
		Run lastText = null;
		for (Run r : runs) {
			if (r.text != null) {
				lastText = r;
			}
		}
		assertEquals(" is also banned from the game.", lastText.text);
	}
}
