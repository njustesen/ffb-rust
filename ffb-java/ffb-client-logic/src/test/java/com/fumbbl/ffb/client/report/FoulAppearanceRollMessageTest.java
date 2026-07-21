package com.fumbbl.ffb.client.report;

import com.fumbbl.ffb.model.Player;
import com.fumbbl.ffb.report.ReportFoulAppearanceRoll;
import org.junit.jupiter.api.Test;
import org.mockito.Mock;

import java.util.List;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.BDDMockito.given;

class FoulAppearanceRollMessageTest extends ReportMessageTestBase {

	@SuppressWarnings("rawtypes")
	@Mock
	private Player attacker;

	@SuppressWarnings("rawtypes")
	@Mock
	private Player defender;

	@Test
	public void successfulRollWithoutDefender() {
		given(game.getActingPlayer().getPlayer()).willReturn(attacker);
		given(attacker.getName()).willReturn("attacker");
		given(game.getTeamHome().hasPlayer(attacker)).willReturn(true);

		ReportFoulAppearanceRoll report = new ReportFoulAppearanceRoll("attacker", true, 4, 2, false, null);
		List<Run> runs = render(new FoulAppearanceRollMessage(), report);

		assertTrue(runs.stream().anyMatch(r -> r.text != null && r.text.contains("resists the Foul Appearance")));
		assertFalse(runs.stream().anyMatch(r -> " of ".equals(r.text)));
	}

	@Test
	public void unsuccessfulRollWithDefender() {
		given(game.getActingPlayer().getPlayer()).willReturn(attacker);
		given(attacker.getName()).willReturn("attacker");
		given(game.getTeamHome().hasPlayer(attacker)).willReturn(true);
		given(game.getPlayerById("defender")).willReturn(defender);
		given(defender.getName()).willReturn("defender");
		given(game.getTeamHome().hasPlayer(defender)).willReturn(false);

		ReportFoulAppearanceRoll report = new ReportFoulAppearanceRoll("attacker", false, 1, 3, false, null,
			"defender");
		List<Run> runs = render(new FoulAppearanceRollMessage(), report);

		assertTrue(runs.stream().anyMatch(r -> r.text != null && r.text.contains("cannot overcome the Foul Appearance")));
		assertTrue(runs.stream().anyMatch(r -> " of ".equals(r.text)));
		assertTrue(runs.stream().anyMatch(r -> "defender".equals(r.text)));
	}

	@Test
	public void missingDefenderIdSkipsOfClause() {
		given(game.getActingPlayer().getPlayer()).willReturn(attacker);
		given(attacker.getName()).willReturn("attacker");
		given(game.getTeamHome().hasPlayer(attacker)).willReturn(true);

		ReportFoulAppearanceRoll report = new ReportFoulAppearanceRoll("attacker", false, 1, 3, false, null);
		List<Run> runs = render(new FoulAppearanceRollMessage(), report);

		assertFalse(runs.stream().anyMatch(r -> " of ".equals(r.text)));
	}

	@Test
	public void endsWithPeriod() {
		given(game.getActingPlayer().getPlayer()).willReturn(attacker);
		given(attacker.getName()).willReturn("attacker");
		given(game.getTeamHome().hasPlayer(attacker)).willReturn(true);

		ReportFoulAppearanceRoll report = new ReportFoulAppearanceRoll("attacker", true, 4, 2, false, null);
		List<Run> runs = render(new FoulAppearanceRollMessage(), report);

		String lastText = null;
		for (Run r : runs) {
			if (r.text != null) {
				lastText = r.text;
			}
		}
		assertEquals(".", lastText);
	}

	@Test
	public void reportIdIsFoulAppearanceRoll() {
		assertEquals("foulAppearanceRoll", new FoulAppearanceRollMessage().getKey());
	}
}
