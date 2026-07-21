package com.fumbbl.ffb.client.report;

import com.fumbbl.ffb.FactoryType;
import com.fumbbl.ffb.PlayerGender;
import com.fumbbl.ffb.mechanics.Mechanic;
import com.fumbbl.ffb.mechanics.bb2025.AgilityMechanic;
import com.fumbbl.ffb.model.Player;
import com.fumbbl.ffb.report.ReportJumpRoll;
import org.junit.jupiter.api.Test;
import org.mockito.Mock;

import java.util.List;

import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.BDDMockito.given;

class JumpRollMessageTest extends ReportMessageTestBase {

	@SuppressWarnings("rawtypes")
	@Mock
	private Player player;

	private void stubMechanic() {
		given(game.getRules().getFactory(FactoryType.Factory.MECHANIC).forName(Mechanic.Type.AGILITY.name()))
			.willReturn(new AgilityMechanic());
	}

	private void stubBb2016Mechanic() {
		given(game.getRules().getFactory(FactoryType.Factory.MECHANIC).forName(Mechanic.Type.AGILITY.name()))
			.willReturn(new com.fumbbl.ffb.mechanics.bb2016.AgilityMechanic());
	}

	@Test
	public void successfulJumpReportsJumpsOverOpponents() {
		given(game.getActingPlayer().getPlayer()).willReturn(player);
		given(player.getPlayerGender()).willReturn(PlayerGender.MALE);
		given(game.getTeamHome().hasPlayer(player)).willReturn(true);
		stubMechanic();

		ReportJumpRoll report = new ReportJumpRoll("p1", true, 5, 3, false, null);
		List<Run> runs = render(new JumpRollMessage(), report);

		assertTrue(runs.stream().anyMatch(r -> r.text != null && r.text.contains("jumps over")));
		assertTrue(runs.stream().anyMatch(r -> r.text != null && r.text.startsWith("Succeeded on a roll of 3+")));
	}

	@Test
	public void unsuccessfulJumpReportsTrips() {
		given(game.getActingPlayer().getPlayer()).willReturn(player);
		given(game.getTeamHome().hasPlayer(player)).willReturn(true);
		stubMechanic();

		ReportJumpRoll report = new ReportJumpRoll("p1", false, 1, 4, false, null);
		List<Run> runs = render(new JumpRollMessage(), report);

		assertTrue(runs.stream().anyMatch(r -> " trips while jumping.".equals(r.text)));
		assertTrue(runs.stream().anyMatch(r -> r.text != null && r.text.startsWith("Roll a 4+ to succeed")));
	}

	@Test
	public void reRolledSkipsNeededRollLine() {
		given(game.getActingPlayer().getPlayer()).willReturn(player);
		given(player.getPlayerGender()).willReturn(PlayerGender.MALE);
		given(game.getTeamHome().hasPlayer(player)).willReturn(true);

		ReportJumpRoll report = new ReportJumpRoll("p1", true, 5, 3, true, null);
		List<Run> runs = render(new JumpRollMessage(), report);

		assertFalse(runs.stream().anyMatch(r -> r.text != null && r.text.startsWith("Succeeded on a roll of")));
	}

	@Test
	public void neededRollIncludesAgilityFormula() {
		given(game.getActingPlayer().getPlayer()).willReturn(player);
		given(player.getAgilityWithModifiers()).willReturn(4);
		given(game.getTeamHome().hasPlayer(player)).willReturn(true);
		stubMechanic();

		ReportJumpRoll report = new ReportJumpRoll("p1", false, 1, 3, false, null);
		List<Run> runs = render(new JumpRollMessage(), report);

		assertTrue(runs.stream().anyMatch(r -> r.text != null && r.text.contains("(Roll >= 4+)")));
	}

	@Test
	public void bb2016NeededRollUsesAgFormulaNotBb2025RollFormula() {
		given(game.getActingPlayer().getPlayer()).willReturn(player);
		given(player.getAgilityWithModifiers()).willReturn(4);
		given(game.getTeamHome().hasPlayer(player)).willReturn(true);
		stubBb2016Mechanic();

		ReportJumpRoll report = new ReportJumpRoll("p1", false, 1, 3, false, null);
		List<Run> runs = render(new JumpRollMessage(), report);

		assertTrue(runs.stream().anyMatch(r -> r.text != null && r.text.contains("(AG 4 + Roll > 6).")));
		assertFalse(runs.stream().anyMatch(r -> r.text != null && r.text.contains("(Roll >=")));
	}
}
