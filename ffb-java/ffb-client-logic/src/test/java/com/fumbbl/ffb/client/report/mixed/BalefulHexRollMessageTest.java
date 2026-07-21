package com.fumbbl.ffb.client.report.mixed;

import com.fumbbl.ffb.client.report.ReportMessageTestBase;
import com.fumbbl.ffb.model.Player;
import com.fumbbl.ffb.report.mixed.ReportBalefulHexRoll;
import org.junit.jupiter.api.Test;
import org.mockito.Mock;

import java.util.List;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.mockito.BDDMockito.given;

class BalefulHexRollMessageTest extends ReportMessageTestBase {

	@SuppressWarnings("rawtypes")
	@Mock
	private Player attacker;

	@SuppressWarnings("rawtypes")
	@Mock
	private Player target;

	@Test
	public void successfulMakesTargetMissTurn() {
		given(game.getActingPlayer().getPlayer()).willReturn(attacker);
		given(attacker.getName()).willReturn("Hexer");
		given(game.getTeamHome().hasPlayer(attacker)).willReturn(true);
		given(game.getPlayerById("p2")).willReturn(target);
		given(target.getName()).willReturn("Victim");
		given(game.getTeamHome().hasPlayer(target)).willReturn(false);

		ReportBalefulHexRoll report = new ReportBalefulHexRoll("p1", "p2", true, 5, false);
		List<Run> runs = render(new BalefulHexRollMessage(), report);

		assertEquals("Baleful Hex Roll [ 5 ]", runs.get(0).text);
		assertEquals("Hexer", runs.get(2).text);
		assertEquals(" makes ", runs.get(3).text);
		assertEquals("Victim", runs.get(4).text);
		assertEquals(" miss a turn.", runs.get(5).text);
	}

	@Test
	public void unsuccessfulFailsToMakeTargetMissTurn() {
		given(game.getActingPlayer().getPlayer()).willReturn(attacker);
		given(attacker.getName()).willReturn("Hexer");
		given(game.getTeamHome().hasPlayer(attacker)).willReturn(true);
		given(game.getPlayerById("p2")).willReturn(target);
		given(target.getName()).willReturn("Victim");
		given(game.getTeamHome().hasPlayer(target)).willReturn(false);

		ReportBalefulHexRoll report = new ReportBalefulHexRoll("p1", "p2", false, 1, false);
		List<Run> runs = render(new BalefulHexRollMessage(), report);

		assertEquals(" fails to make ", runs.get(3).text);
	}
}
