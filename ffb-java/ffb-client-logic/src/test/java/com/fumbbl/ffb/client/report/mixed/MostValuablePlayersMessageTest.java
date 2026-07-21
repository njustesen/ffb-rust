package com.fumbbl.ffb.client.report.mixed;

import com.fumbbl.ffb.PlayerGender;
import com.fumbbl.ffb.client.TextStyle;
import com.fumbbl.ffb.client.report.ReportMessageTestBase;
import com.fumbbl.ffb.model.Player;
import com.fumbbl.ffb.report.ReportMostValuablePlayers;
import org.junit.jupiter.api.Test;
import org.mockito.Mock;

import java.util.List;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.mockito.BDDMockito.given;

class MostValuablePlayersMessageTest extends ReportMessageTestBase {

	@SuppressWarnings("rawtypes")
	@Mock
	private Player homer;

	@SuppressWarnings("rawtypes")
	@Mock
	private Player awey;

	@Test
	public void homeWinsReportsMvpForBothSides() {
		given(game.getGameResult().getTeamResultHome().getScore()).willReturn(2);
		given(game.getTeamHome().getName()).willReturn("Team home");
		given(game.getPlayerById("h1")).willReturn(homer);
		given(game.getPlayerById("a1")).willReturn(awey);
		given(homer.getName()).willReturn("Homer");
		given(homer.getPlayerGender()).willReturn(PlayerGender.MALE);
		given(awey.getName()).willReturn("Awey");
		given(awey.getPlayerGender()).willReturn(PlayerGender.MALE);

		ReportMostValuablePlayers report = new ReportMostValuablePlayers();
		report.addPlayerIdHome("h1");
		report.addPlayerIdAway("a1");
		List<Run> runs = render(new MostValuablePlayersMessage(), report);

		assertEquals("Team home win the game.", runs.get(0).text);
		assertEquals(TextStyle.TURN_HOME, runs.get(0).textStyle);
		boolean hasHomer = runs.stream().anyMatch(r -> "Homer".equals(r.text));
		boolean hasAwey = runs.stream().anyMatch(r -> "Awey".equals(r.text));
		assertEquals(true, hasHomer);
		assertEquals(true, hasAwey);
	}

	@Test
	public void tieReportsTieMessage() {
		ReportMostValuablePlayers report = new ReportMostValuablePlayers();
		List<Run> runs = render(new MostValuablePlayersMessage(), report);

		assertEquals("The game ends in a tie.", runs.get(0).text);
		assertEquals(TextStyle.TURN, runs.get(0).textStyle);
	}

	@Test
	public void homeConcededLegally() {
		given(game.getGameResult().getTeamResultHome().hasConceded()).willReturn(true);
		given(game.isConcededLegally()).willReturn(true);
		given(game.getTeamHome().getCoach()).willReturn("CoachHome");

		ReportMostValuablePlayers report = new ReportMostValuablePlayers();
		List<Run> runs = render(new MostValuablePlayersMessage(), report);

		assertEquals("Coach CoachHome concedes the game without penalties due to excessive player loss.", runs.get(0).text);
		assertEquals(TextStyle.TURN_HOME, runs.get(0).textStyle);
	}

	@Test
	public void awayConcededWithoutLegalFlag() {
		given(game.getGameResult().getTeamResultAway().hasConceded()).willReturn(true);
		given(game.getTeamAway().getCoach()).willReturn("CoachAway");

		ReportMostValuablePlayers report = new ReportMostValuablePlayers();
		List<Run> runs = render(new MostValuablePlayersMessage(), report);

		assertEquals("Coach CoachAway concedes the game.", runs.get(0).text);
		assertEquals(TextStyle.TURN_AWAY, runs.get(0).textStyle);
	}

	@Test
	public void awayWinsOnScoreDiff() {
		given(game.getGameResult().getTeamResultAway().getScore()).willReturn(3);
		given(game.getTeamAway().getName()).willReturn("Team away");

		ReportMostValuablePlayers report = new ReportMostValuablePlayers();
		List<Run> runs = render(new MostValuablePlayersMessage(), report);

		assertEquals("Team away win the game.", runs.get(0).text);
		assertEquals(TextStyle.TURN_AWAY, runs.get(0).textStyle);
	}
}
