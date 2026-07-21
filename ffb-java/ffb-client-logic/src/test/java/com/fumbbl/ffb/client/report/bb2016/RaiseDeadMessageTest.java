package com.fumbbl.ffb.client.report.bb2016;

import com.fumbbl.ffb.FactoryType;
import com.fumbbl.ffb.client.report.ReportMessageTestBase;
import com.fumbbl.ffb.mechanics.Mechanic;
import com.fumbbl.ffb.mechanics.bb2016.InjuryMechanic;
import com.fumbbl.ffb.model.Player;
import com.fumbbl.ffb.report.ReportRaiseDead;
import org.junit.jupiter.api.Test;
import org.mockito.Mock;

import java.util.List;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.BDDMockito.given;

class RaiseDeadMessageTest extends ReportMessageTestBase {

	@SuppressWarnings("rawtypes")
	@Mock
	private Player raisedPlayer;

	@Test
	public void getKeyIsRaiseDead() {
		assertEquals("raiseDead", new RaiseDeadMessage().getKey());
	}

	@Test
	public void zombieReportsJoinTeam() {
		given(game.getPlayerById("p1")).willReturn(raisedPlayer);
		given(raisedPlayer.getName()).willReturn("Grubb");
		given(game.getTeamHome().hasPlayer(raisedPlayer)).willReturn(true);
		given(game.getTeamHome().getName()).willReturn("Team home");

		ReportRaiseDead report = new ReportRaiseDead("p1", null, false);
		List<Run> runs = render(new RaiseDeadMessage(), report);

		assertTrue(runs.stream().anyMatch(r -> " is raised from the dead to join team ".equals(r.text)));
		assertTrue(runs.stream().anyMatch(r -> " as a Zombie.".equals(r.text)));
	}

	@Test
	public void nurglesRotReportsRotter() {
		given(game.getPlayerById("p1")).willReturn(raisedPlayer);
		given(raisedPlayer.getName()).willReturn("Grubb");
		given(game.getTeamHome().hasPlayer(raisedPlayer)).willReturn(true);
		given(game.getTeamHome().getName()).willReturn("Team home");
		given(game.getFactory(FactoryType.Factory.MECHANIC).forName(Mechanic.Type.INJURY.name()))
			.willReturn(new InjuryMechanic());

		ReportRaiseDead report = new ReportRaiseDead("p1", null, true);
		List<Run> runs = render(new RaiseDeadMessage(), report);

		assertTrue(runs.stream().anyMatch(r -> " as a Rotter in the next game.".equals(r.text)));
	}
}
