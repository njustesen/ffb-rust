package com.fumbbl.ffb.client.report.bb2025;

import com.fumbbl.ffb.FactoryType;
import com.fumbbl.ffb.client.TextStyle;
import com.fumbbl.ffb.client.report.ReportMessageTestBase;
import com.fumbbl.ffb.mechanics.InjuryMechanic;
import com.fumbbl.ffb.mechanics.Mechanic;
import com.fumbbl.ffb.model.Player;
import com.fumbbl.ffb.report.ReportId;
import com.fumbbl.ffb.report.ReportRaiseDead;
import org.junit.jupiter.api.Test;
import org.mockito.Mock;

import java.util.List;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.BDDMockito.given;

class RaiseDeadMessageTest extends ReportMessageTestBase {

	@SuppressWarnings("rawtypes")
	@Mock
	private Player zombieOne;

	@SuppressWarnings("rawtypes")
	@Mock
	private Player zombieTwo;

	@Mock
	private InjuryMechanic injuryMechanic;

	@Test
	public void reportIdIsRaiseDead() {
		assertEquals(ReportId.RAISE_DEAD.getKey(), new RaiseDeadMessage().getKey());
	}

	@Test
	public void raisesHomePlayerWithoutNurglesRot() {
		given(game.getPlayerById("p1")).willReturn(zombieOne);
		given(game.getTeamHome().hasPlayer(zombieOne)).willReturn(true);
		given(game.getTeamHome().getName()).willReturn("Team home");
		given(game.getTeamAway().getName()).willReturn("Team away");

		ReportRaiseDead report = new ReportRaiseDead("p1", "Zombie", false);
		List<Run> runs = render(new RaiseDeadMessage(), report);

		assertTrue(runs.stream().anyMatch(r -> " is raised from the dead to join team ".equals(r.text)));
		assertTrue(runs.stream().anyMatch(r -> "Team home".equals(r.text)));
		assertTrue(runs.stream().anyMatch(r -> " as a Zombie.".equals(r.text)));
	}

	@Test
	public void raisesAwayPlayerWithNurglesRot() {
		given(game.getPlayerById("p2")).willReturn(zombieTwo);
		given(game.getTeamHome().hasPlayer(zombieTwo)).willReturn(false);
		given(game.getTeamHome().getName()).willReturn("Team home");
		given(game.getTeamAway().getName()).willReturn("Team away");
		given(game.getFactory(FactoryType.Factory.MECHANIC).forName(Mechanic.Type.INJURY.name())).willReturn(injuryMechanic);
		given(injuryMechanic.raisedByNurgleMessage()).willReturn(" is now Plague Ridden and will join team ");

		ReportRaiseDead report = new ReportRaiseDead("p2", "Rotter", true);
		List<Run> runs = render(new RaiseDeadMessage(), report);

		assertTrue(runs.stream().anyMatch(r -> "Team away".equals(r.text)));
		assertTrue(runs.stream().anyMatch(r -> " as a Rotter.".equals(r.text)));
		assertFalse(runs.stream().anyMatch(r -> " is raised from the dead to join team ".equals(r.text)));
	}

	@Test
	public void textStyleReflectsHomeVsAway() {
		given(game.getPlayerById("p1")).willReturn(zombieOne);
		given(game.getTeamHome().hasPlayer(zombieOne)).willReturn(true);
		given(game.getTeamHome().getName()).willReturn("Team home");
		given(game.getTeamAway().getName()).willReturn("Team away");

		ReportRaiseDead report = new ReportRaiseDead("p1", "Zombie", false);
		List<Run> runs = render(new RaiseDeadMessage(), report);

		Run homeRun = runs.stream().filter(r -> "Team home".equals(r.text)).findFirst().orElseThrow();
		assertEquals(TextStyle.HOME, homeRun.textStyle);
	}
}
