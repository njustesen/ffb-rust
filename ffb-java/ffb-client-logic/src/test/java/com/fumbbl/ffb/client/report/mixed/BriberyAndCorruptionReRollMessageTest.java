package com.fumbbl.ffb.client.report.mixed;

import com.fumbbl.ffb.client.TextStyle;
import com.fumbbl.ffb.client.report.ReportMessageTestBase;
import com.fumbbl.ffb.inducement.BriberyAndCorruptionAction;
import com.fumbbl.ffb.model.Team;
import com.fumbbl.ffb.report.mixed.ReportBriberyAndCorruptionReRoll;
import org.junit.jupiter.api.Test;

import java.util.List;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.mockito.BDDMockito.given;

class BriberyAndCorruptionReRollMessageTest extends ReportMessageTestBase {

	@Test
	public void usedActionPrintsUsedMessage() {
		Team homeTeam = game.getTeamHome();
		given(game.getTeamById("home")).willReturn(homeTeam);
		given(homeTeam.getName()).willReturn("Home Team");

		ReportBriberyAndCorruptionReRoll report = new ReportBriberyAndCorruptionReRoll("home", BriberyAndCorruptionAction.USED);
		List<Run> runs = render(new BriberyAndCorruptionReRollMessage(), report);

		assertEquals("Home Team", runs.get(0).text);
		assertEquals(TextStyle.HOME_BOLD, runs.get(0).textStyle);
		assertEquals(" use Bribery and Corruption to re-roll their Argue the Call roll.", runs.get(1).text);
	}

	@Test
	public void addedActionPrintsAddedMessage() {
		Team awayTeam = game.getTeamAway();
		given(game.getTeamById("away")).willReturn(awayTeam);
		given(awayTeam.getName()).willReturn("Away Team");

		ReportBriberyAndCorruptionReRoll report = new ReportBriberyAndCorruptionReRoll("away", BriberyAndCorruptionAction.ADDED);
		List<Run> runs = render(new BriberyAndCorruptionReRollMessage(), report);

		assertEquals(TextStyle.AWAY_BOLD, runs.get(0).textStyle);
		assertEquals(
			" may re-roll a natural 1 on an Argue the Call roll once in this game due to Bribery and Corruption.",
			runs.get(1).text);
	}

	@Test
	public void wastedActionPrintsWastedMessage() {
		Team homeTeam = game.getTeamHome();
		given(game.getTeamById("home")).willReturn(homeTeam);
		given(homeTeam.getName()).willReturn("Home Team");

		ReportBriberyAndCorruptionReRoll report = new ReportBriberyAndCorruptionReRoll("home", BriberyAndCorruptionAction.WASTED);
		List<Run> runs = render(new BriberyAndCorruptionReRollMessage(), report);

		assertEquals(
			" have no use for their Bribery and Corruption as the coach was banned for more than one argue.",
			runs.get(1).text);
	}
}
