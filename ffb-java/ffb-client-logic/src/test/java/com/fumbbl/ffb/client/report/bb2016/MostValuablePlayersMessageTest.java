package com.fumbbl.ffb.client.report.bb2016;

import com.fumbbl.ffb.PlayerGender;
import com.fumbbl.ffb.client.report.ReportMessageTestBase;
import com.fumbbl.ffb.model.Player;
import com.fumbbl.ffb.report.ReportMostValuablePlayers;
import org.junit.jupiter.api.Test;
import org.mockito.Mock;

import java.util.List;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.BDDMockito.given;

class MostValuablePlayersMessageTest extends ReportMessageTestBase {

	@SuppressWarnings("rawtypes")
	@Mock
	private Player grubb;

	@Test
	public void getKeyIsMostValuablePlayers() {
		assertEquals("mostValuablePlayers", new MostValuablePlayersMessage().getKey());
	}

	@Test
	public void homeWinReportsWinAndMvp() {
		given(game.getGameResult().getTeamResultHome().getScore()).willReturn(2);
		given(game.getTeamHome().getName()).willReturn("Team home");
		given(game.getPlayerById("p1")).willReturn(grubb);
		given(grubb.getName()).willReturn("Grubb");
		given(grubb.getPlayerGender()).willReturn(PlayerGender.MALE);

		ReportMostValuablePlayers report = new ReportMostValuablePlayers();
		report.addPlayerIdHome("p1");
		List<Run> runs = render(new MostValuablePlayersMessage(), report);

		assertTrue(runs.stream().anyMatch(r -> "Team home win the game.".equals(r.text)));
		assertTrue(runs.stream().anyMatch(r -> "Grubb".equals(r.text)));
	}

	@Test
	public void tieReportsTieMessage() {
		ReportMostValuablePlayers report = new ReportMostValuablePlayers();
		List<Run> runs = render(new MostValuablePlayersMessage(), report);

		assertTrue(runs.stream().anyMatch(r -> "The game ends in a tie.".equals(r.text)));
	}

	@Test
	public void concededReportsConcessionMessage() {
		given(game.getGameResult().getTeamResultAway().hasConceded()).willReturn(true);
		given(game.getTeamAway().getCoach()).willReturn("Coachaway");

		ReportMostValuablePlayers report = new ReportMostValuablePlayers();
		List<Run> runs = render(new MostValuablePlayersMessage(), report);

		assertTrue(runs.stream().anyMatch(r -> "Coach Coachaway concedes the game.".equals(r.text)));
	}
}
