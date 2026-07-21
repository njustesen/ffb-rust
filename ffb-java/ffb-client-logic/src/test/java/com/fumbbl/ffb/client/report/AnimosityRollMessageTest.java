package com.fumbbl.ffb.client.report;

import com.fumbbl.ffb.PlayerGender;
import com.fumbbl.ffb.client.TextStyle;
import com.fumbbl.ffb.model.Player;
import com.fumbbl.ffb.report.ReportAnimosityRoll;
import org.junit.jupiter.api.Test;
import org.mockito.Mock;

import java.util.List;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.BDDMockito.given;

class AnimosityRollMessageTest extends ReportMessageTestBase {

	@SuppressWarnings("rawtypes")
	@Mock
	private Player player;

	@Test
	public void successNotRerolledShowsNeededRoll() {
		given(game.getActingPlayer().getPlayer()).willReturn(player);
		given(player.getName()).willReturn("Player");
		given(player.getPlayerGender()).willReturn(PlayerGender.MALE);
		given(game.getTeamHome().hasPlayer(player)).willReturn(true);

		ReportAnimosityRoll report = new ReportAnimosityRoll("p1", true, 4, 2, false, null);
		List<Run> runs = render(new AnimosityRollMessage(), report);

		assertEquals("Animosity Roll [ 4 ]", runs.get(0).text);
		assertEquals(" resists his Animosity.", runs.get(3).text);
		Run needed = runs.stream().filter(r -> r.textStyle == TextStyle.NEEDED_ROLL).findFirst().orElseThrow();
		assertEquals("Succeeded on a roll of 2+", needed.text);
	}

	@Test
	public void failureRerolledHidesNeededRoll() {
		given(game.getActingPlayer().getPlayer()).willReturn(player);
		given(player.getName()).willReturn("Player");
		given(player.getPlayerGender()).willReturn(PlayerGender.FEMALE);
		given(game.getTeamHome().hasPlayer(player)).willReturn(true);

		ReportAnimosityRoll report = new ReportAnimosityRoll("p1", false, 2, 3, true, null);
		List<Run> runs = render(new AnimosityRollMessage(), report);

		assertEquals(" gives in to her Animosity.", runs.get(3).text);
		assertTrue(runs.stream().noneMatch(r -> r.textStyle == TextStyle.NEEDED_ROLL));
	}

	@Test
	public void failureNotRerolledShowsNeededRoll() {
		given(game.getActingPlayer().getPlayer()).willReturn(player);
		given(player.getName()).willReturn("Player");
		given(player.getPlayerGender()).willReturn(PlayerGender.NONBINARY);
		given(game.getTeamHome().hasPlayer(player)).willReturn(true);

		ReportAnimosityRoll report = new ReportAnimosityRoll("p1", false, 2, 5, false, null);
		List<Run> runs = render(new AnimosityRollMessage(), report);

		assertEquals(" gives in to their Animosity.", runs.get(3).text);
		Run needed = runs.stream().filter(r -> r.textStyle == TextStyle.NEEDED_ROLL).findFirst().orElseThrow();
		assertEquals("Roll a 5+ to succeed", needed.text);
	}
}
