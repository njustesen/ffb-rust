package com.fumbbl.ffb.client.report;

import com.fumbbl.ffb.client.TextStyle;
import com.fumbbl.ffb.model.Player;
import com.fumbbl.ffb.report.ReportFoul;
import org.junit.jupiter.api.Test;
import org.mockito.Mock;

import java.util.ArrayList;
import java.util.Arrays;
import java.util.List;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.mockito.BDDMockito.given;

class FoulMessageTest extends ReportMessageTestBase {

	@SuppressWarnings("rawtypes")
	@Mock
	private Player attacker;

	@SuppressWarnings("rawtypes")
	@Mock
	private Player defender;

	private List<String> texts(List<Run> runs) {
		List<String> texts = new ArrayList<>();
		for (Run run : runs) {
			if (run.text != null) {
				texts.add(run.text);
			}
		}
		return texts;
	}

	private void stubPlayers() {
		given(game.getActingPlayer().getPlayer()).willReturn(attacker);
		given(attacker.getName()).willReturn("attacker");
		given(game.getPlayerById("defender")).willReturn(defender);
		given(defender.getName()).willReturn("defender");
		given(game.getTeamHome().hasPlayer(attacker)).willReturn(true);
		given(game.getTeamHome().hasPlayer(defender)).willReturn(false);
	}

	@Test
	public void rendersAttackerFoulsDefender() {
		stubPlayers();

		ReportFoul report = new ReportFoul("defender");
		List<Run> runs = render(new FoulMessage(), report);

		assertEquals(Arrays.asList("attacker", " fouls ", "defender", ":"), texts(runs));
	}

	@Test
	public void incrementsIndentAfterRender() {
		stubPlayers();

		ReportFoul report = new ReportFoul("defender");
		assertEquals(0, statusReport.getIndent());
		render(new FoulMessage(), report);
		assertEquals(1, statusReport.getIndent());
	}

	@Test
	public void attackerAndDefenderUseHomeAwayStyles() {
		stubPlayers();

		ReportFoul report = new ReportFoul("defender");
		List<Run> runs = render(new FoulMessage(), report);

		assertEquals(TextStyle.HOME_BOLD, runs.get(0).textStyle);
		assertEquals(TextStyle.AWAY_BOLD, runs.get(2).textStyle);
	}
}
