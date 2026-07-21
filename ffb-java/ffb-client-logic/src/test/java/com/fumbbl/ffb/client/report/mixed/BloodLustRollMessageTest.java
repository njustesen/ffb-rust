package com.fumbbl.ffb.client.report.mixed;

import com.fumbbl.ffb.client.report.ReportMessageTestBase;
import com.fumbbl.ffb.model.Player;
import com.fumbbl.ffb.modifiers.RollModifier;
import com.fumbbl.ffb.report.ReportBloodLustRoll;
import org.junit.jupiter.api.Test;
import org.mockito.Mock;

import java.util.List;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.mockito.BDDMockito.given;

class BloodLustRollMessageTest extends ReportMessageTestBase {

	@SuppressWarnings("rawtypes")
	@Mock
	private Player player;

	@Test
	public void successWithoutReRollShowsNeededRoll() {
		given(game.getActingPlayer().getPlayer()).willReturn(player);
		given(player.getName()).willReturn("Vampire");
		given(game.getTeamHome().hasPlayer(player)).willReturn(true);

		ReportBloodLustRoll report = new ReportBloodLustRoll(null, true, 4, 2, false, new RollModifier<?>[0]);
		List<Run> runs = render(new BloodLustRollMessage(), report);

		assertEquals("Blood Lust Roll [ 4 ]", runs.get(0).text);
		assertEquals("Vampire", runs.get(2).text);
		assertEquals(" resists the Blood Lust.", runs.get(3).text);
		assertEquals("Succeeded on a roll of 2+", runs.get(5).text);
	}

	@Test
	public void failurePrintsFeedWarningAndNeededRoll() {
		given(game.getActingPlayer().getPlayer()).willReturn(player);
		given(player.getName()).willReturn("Vampire");
		given(game.getTeamHome().hasPlayer(player)).willReturn(true);

		ReportBloodLustRoll report = new ReportBloodLustRoll(null, false, 1, 2, false, new RollModifier<?>[0]);
		List<Run> runs = render(new BloodLustRollMessage(), report);

		assertEquals(" gives in to the Blood Lust.", runs.get(3).text);
		assertEquals(
			"Player must feed at the end of the action or lose tackle zone (and drop ball if carrying) and suffer a turnover.",
			runs.get(5).text
		);
		assertEquals("Roll a 2+ to succeed", runs.get(7).text);
	}

	@Test
	public void reRolledSkipsNeededRoll() {
		given(game.getActingPlayer().getPlayer()).willReturn(player);
		given(player.getName()).willReturn("Vampire");
		given(game.getTeamHome().hasPlayer(player)).willReturn(true);

		ReportBloodLustRoll report = new ReportBloodLustRoll(null, true, 4, 2, true, new RollModifier<?>[0]);
		List<Run> runs = render(new BloodLustRollMessage(), report);

		// roll println (2) + player print (1) + resists println (2) = 5, no needed-roll line.
		assertEquals(5, runs.size());
	}
}
