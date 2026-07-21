package com.fumbbl.ffb.client.report;

import com.fumbbl.ffb.FactoryType;
import com.fumbbl.ffb.PlayerGender;
import com.fumbbl.ffb.client.TextStyle;
import com.fumbbl.ffb.mechanics.Mechanic;
import com.fumbbl.ffb.mechanics.bb2025.AgilityMechanic;
import com.fumbbl.ffb.model.Player;
import com.fumbbl.ffb.report.ReportJumpUpRoll;
import org.junit.jupiter.api.Test;
import org.mockito.Mock;

import java.util.List;

import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.BDDMockito.given;

class JumpUpRollMessageTest extends ReportMessageTestBase {

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
	public void successfulNotRerolledPrintsNeededRoll() {
		given(game.getActingPlayer().getPlayer()).willReturn(player);
		given(player.getPlayerGender()).willReturn(PlayerGender.MALE);
		given(player.getAgilityWithModifiers()).willReturn(3);
		given(game.getTeamHome().hasPlayer(player)).willReturn(true);
		stubMechanic();

		ReportJumpUpRoll report = new ReportJumpUpRoll("p1", true, 5, 3, false, null);
		List<Run> runs = render(new JumpUpRollMessage(), report);

		assertTrue(runs.stream().anyMatch(r -> "Jump Up Roll [ 5 ]".equals(r.text)));
		assertTrue(runs.stream().anyMatch(r -> " jumps up to block his opponent.".equals(r.text)));
		assertTrue(runs.stream().anyMatch(r -> r.text != null && r.text.startsWith("Succeeded on a roll of 3+")));
	}

	@Test
	public void failedNotRerolledPrintsNeededRoll() {
		given(game.getActingPlayer().getPlayer()).willReturn(player);
		given(player.getPlayerGender()).willReturn(PlayerGender.MALE);
		given(player.getAgilityWithModifiers()).willReturn(3);
		given(game.getTeamHome().hasPlayer(player)).willReturn(true);
		stubMechanic();

		ReportJumpUpRoll report = new ReportJumpUpRoll("p1", false, 1, 3, false, null);
		List<Run> runs = render(new JumpUpRollMessage(), report);

		assertTrue(runs.stream().anyMatch(r -> " doesn't get to his feet.".equals(r.text)));
		assertTrue(runs.stream().anyMatch(r -> r.text != null && r.text.startsWith("Roll a 3+ to succeed")));
	}

	@Test
	public void reRolledSuppressesNeededRollLine() {
		given(game.getActingPlayer().getPlayer()).willReturn(player);
		given(player.getPlayerGender()).willReturn(PlayerGender.MALE);
		given(game.getTeamHome().hasPlayer(player)).willReturn(true);

		ReportJumpUpRoll report = new ReportJumpUpRoll("p1", true, 5, 3, true, null);
		List<Run> runs = render(new JumpUpRollMessage(), report);

		assertTrue(runs.stream().noneMatch(r -> r.textStyle == TextStyle.NEEDED_ROLL));
	}

	@Test
	public void bb2016NeededRollUsesAgFormulaNotBb2025RollFormula() {
		given(game.getActingPlayer().getPlayer()).willReturn(player);
		given(player.getPlayerGender()).willReturn(PlayerGender.MALE);
		given(player.getAgilityWithModifiers()).willReturn(3);
		given(game.getTeamHome().hasPlayer(player)).willReturn(true);
		stubBb2016Mechanic();

		ReportJumpUpRoll report = new ReportJumpUpRoll("p1", false, 1, 3, false, null);
		List<Run> runs = render(new JumpUpRollMessage(), report);

		assertTrue(runs.stream().anyMatch(r -> r.text != null && r.text.contains("(AG 3 + Roll > 6).")));
		assertFalse(runs.stream().anyMatch(r -> r.text != null && r.text.contains("(Roll >=")));
	}
}
