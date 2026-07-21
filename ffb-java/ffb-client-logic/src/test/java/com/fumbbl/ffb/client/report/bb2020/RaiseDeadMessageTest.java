package com.fumbbl.ffb.client.report.bb2020;

import com.fumbbl.ffb.FactoryType;
import com.fumbbl.ffb.client.TextStyle;
import com.fumbbl.ffb.client.report.ReportMessageTestBase;
import com.fumbbl.ffb.mechanics.Mechanic;
import com.fumbbl.ffb.mechanics.bb2020.InjuryMechanic;
import com.fumbbl.ffb.model.Player;
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
	private Player raisedPlayer;

	@Test
	public void plainZombieJoinHomeTeam() {
		given(game.getPlayerById("raised")).willReturn(raisedPlayer);
		given(game.getTeamHome().hasPlayer(raisedPlayer)).willReturn(true);
		given(game.getTeamHome().getName()).willReturn("Home Team");
		given(game.getTeamHome().getRoster().getRaisedRosterPosition()).willReturn(null);

		ReportRaiseDead report = new ReportRaiseDead("raised", null, false);
		List<Run> runs = render(new RaiseDeadMessage(), report);
		List<String> texts = runs.stream().map(r -> r.text).collect(java.util.stream.Collectors.toList());

		assertTrue(texts.stream().anyMatch(t -> t != null && t.contains("is raised from the dead to join team")));
		assertTrue(texts.contains("Home Team"));
		assertTrue(texts.contains(" as a Zombie."));
	}

	@Test
	public void nurglesRotUsesRotterAndNurgleMessage() {
		given(game.getPlayerById("raised")).willReturn(raisedPlayer);
		given(game.getTeamHome().hasPlayer(raisedPlayer)).willReturn(true);
		given(game.getTeamHome().getName()).willReturn("Home Team");
		given(game.getTeamHome().getRoster().getRaisedRosterPosition()).willReturn(null);
		given(game.getFactory(FactoryType.Factory.MECHANIC).forName(Mechanic.Type.INJURY.name()))
			.willReturn(new InjuryMechanic());

		ReportRaiseDead report = new ReportRaiseDead("raised", null, true);
		List<Run> runs = render(new RaiseDeadMessage(), report);
		List<String> texts = runs.stream().map(r -> r.text).collect(java.util.stream.Collectors.toList());

		assertTrue(texts.contains(" as a Rotter."));
	}

	@Test
	public void awayTeamPlayerUsesAwayStyle() {
		given(game.getPlayerById("raised")).willReturn(raisedPlayer);
		given(game.getTeamHome().hasPlayer(raisedPlayer)).willReturn(false);
		given(game.getTeamAway().getName()).willReturn("Away Team");
		given(game.getTeamAway().getRoster().getRaisedRosterPosition()).willReturn(null);

		ReportRaiseDead report = new ReportRaiseDead("raised", null, false);
		List<Run> runs = render(new RaiseDeadMessage(), report);

		Run awayRun = runs.stream().filter(r -> "Away Team".equals(r.text)).findFirst().orElseThrow();
		assertEquals(TextStyle.AWAY, awayRun.textStyle);
	}

	@Test
	public void unknownPlayerDoesNotPanic() {
		given(game.getPlayerById("ghost")).willReturn(null);

		ReportRaiseDead report = new ReportRaiseDead("ghost", null, false);
		List<Run> runs = render(new RaiseDeadMessage(), report);
		List<String> texts = runs.stream().map(r -> r.text).collect(java.util.stream.Collectors.toList());

		assertFalse(texts.stream().anyMatch(t -> t != null && t.endsWith("Zombie.")));
	}
}
