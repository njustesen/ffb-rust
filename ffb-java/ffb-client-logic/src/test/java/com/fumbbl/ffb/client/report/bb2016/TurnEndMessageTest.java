package com.fumbbl.ffb.client.report.bb2016;

import com.fumbbl.ffb.HeatExhaustion;
import com.fumbbl.ffb.KnockoutRecovery;
import com.fumbbl.ffb.TurnMode;
import com.fumbbl.ffb.client.report.ReportMessageTestBase;
import com.fumbbl.ffb.model.Player;
import com.fumbbl.ffb.report.bb2016.ReportTurnEnd;
import org.junit.jupiter.api.Test;
import org.mockito.Mock;

import java.util.Collections;
import java.util.List;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.BDDMockito.given;

class TurnEndMessageTest extends ReportMessageTestBase {

	@SuppressWarnings("rawtypes")
	@Mock
	private Player p1;

	@Test
	public void getKeyIsTurnEnd() {
		assertEquals("turnEnd", new TurnEndMessage().getKey());
	}

	@Test
	public void touchdownPlayerReportsScore() {
		given(game.getPlayerById("p1")).willReturn(p1);

		ReportTurnEnd report = new ReportTurnEnd("p1", new KnockoutRecovery[0], new HeatExhaustion[0], Collections.emptyList());
		List<Run> runs = render(new TurnEndMessage(), report);

		assertTrue(runs.stream().anyMatch(r -> " scores a touchdown.".equals(r.text)));
	}

	@Test
	public void regularTurnModeReportsTurnStart() {
		given(game.getTurnMode()).willReturn(TurnMode.REGULAR);
		given(game.isHomePlaying()).willReturn(true);
		given(game.getTeamHome().getName()).willReturn("Team home");
		given(game.getTurnDataHome().getTurnNr()).willReturn(3);

		ReportTurnEnd report = new ReportTurnEnd(null, new KnockoutRecovery[0], new HeatExhaustion[0], Collections.emptyList());
		List<Run> runs = render(new TurnEndMessage(), report);

		assertTrue(runs.stream().anyMatch(r -> "Team home start turn 3.".equals(r.text)));
	}

	@Test
	public void knockoutRecoveryReportsRegainingConsciousness() {
		given(game.getPlayerById("p1")).willReturn(p1);

		ReportTurnEnd report = new ReportTurnEnd(null,
			new KnockoutRecovery[]{new KnockoutRecovery("p1", true, 5, 0, null)}, new HeatExhaustion[0], Collections.emptyList());
		List<Run> runs = render(new TurnEndMessage(), report);

		assertTrue(runs.stream().anyMatch(r -> " is regaining consciousness.".equals(r.text)));
	}

	@Test
	public void heatExhaustionReportsExhausted() {
		given(game.getPlayerById("p1")).willReturn(p1);

		ReportTurnEnd report = new ReportTurnEnd(null, new KnockoutRecovery[0],
			new HeatExhaustion[]{new HeatExhaustion("p1", true, 2)}, Collections.emptyList());
		List<Run> runs = render(new TurnEndMessage(), report);

		assertTrue(runs.stream().anyMatch(r -> " is suffering from heat exhaustion.".equals(r.text)));
	}
}
