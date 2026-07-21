package com.fumbbl.ffb.client.report.mixed;

import com.fumbbl.ffb.HeatExhaustion;
import com.fumbbl.ffb.KnockoutRecovery;
import com.fumbbl.ffb.TurnMode;
import com.fumbbl.ffb.client.ParagraphStyle;
import com.fumbbl.ffb.client.TextStyle;
import com.fumbbl.ffb.client.report.ReportMessageTestBase;
import com.fumbbl.ffb.model.Player;
import com.fumbbl.ffb.model.TurnData;
import com.fumbbl.ffb.report.mixed.ReportTurnEnd;
import org.junit.jupiter.api.Test;
import org.mockito.Mock;

import java.util.Collections;
import java.util.List;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNull;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.BDDMockito.given;

class TurnEndMessageTest extends ReportMessageTestBase {

	@SuppressWarnings("rawtypes")
	@Mock
	private Player scorer;

	@SuppressWarnings("rawtypes")
	@Mock
	private Player knocked;

	@SuppressWarnings("rawtypes")
	@Mock
	private Player heated;

	@SuppressWarnings("rawtypes")
	@Mock
	private Player zapped;

	@Test
	public void touchdownAndRegularTurnStartHome() {
		given(game.getPlayerById("scorer")).willReturn(scorer);
		given(scorer.getName()).willReturn("Scorer");
		given(game.getTeamHome().hasPlayer(scorer)).willReturn(true);
		given(game.isHomePlaying()).willReturn(true);
		given(game.getTurnMode()).willReturn(TurnMode.REGULAR);
		given(game.getTeamHome().getName()).willReturn("Team home");
		TurnData turnDataHome = game.getTurnDataHome();
		given(turnDataHome.getTurnNr()).willReturn(7);

		ReportTurnEnd report = new ReportTurnEnd("scorer", new KnockoutRecovery[0], new HeatExhaustion[0], Collections.emptyList(), 0);
		List<Run> runs = render(new TurnEndMessage(), report);

		assertEquals("Scorer", runs.get(0).text);
		assertEquals(TextStyle.HOME_BOLD, runs.get(0).textStyle);
		assertEquals(" scores a touchdown.", runs.get(1).text);
		Run last = runs.get(runs.size() - 1);
		assertNull(last.paragraphStyle); // terminator run from println_style
		Run turnRun = runs.get(runs.size() - 2);
		assertEquals("Team home start turn 7.", turnRun.text);
		assertEquals(TextStyle.TURN_HOME, turnRun.textStyle);
		assertEquals(ParagraphStyle.SPACE_ABOVE_BELOW, turnRun.paragraphStyle);
	}

	@Test
	public void knockoutRecoveryRecoveringAndUnconscious() {
		// playerIdTouchdown is null in this test's reports; without this stub the deep-stub
		// Game mock would auto-vivify a non-null Player mock for getPlayerById(null) and the
		// handler would spuriously render a touchdown block.
		given(game.getPlayerById((String) null)).willReturn(null);
		given(game.getPlayerById("ko1")).willReturn(knocked);
		given(knocked.getName()).willReturn("Knocked");
		given(game.getTeamHome().hasPlayer(knocked)).willReturn(true);

		ReportTurnEnd report = new ReportTurnEnd(null,
			new KnockoutRecovery[] { new KnockoutRecovery("ko1", true, 5, 0, null) },
			new HeatExhaustion[0], Collections.emptyList(), 0);
		List<Run> runs = render(new TurnEndMessage(), report);

		Run playerRun = runs.stream().filter(r -> "Knocked".equals(r.text)).findFirst().orElseThrow();
		int idx = runs.indexOf(playerRun);
		assertEquals(" is regaining consciousness.", runs.get(idx + 1).text);

		ReportTurnEnd report2 = new ReportTurnEnd(null,
			new KnockoutRecovery[] { new KnockoutRecovery("ko1", false, 2, 0, null) },
			new HeatExhaustion[0], Collections.emptyList(), 0);
		List<Run> runs2 = render(new TurnEndMessage(), report2);
		// The log mock captures appends cumulatively across both render() calls, so take the
		// LAST "Knocked" run (from the second render) rather than the first.
		Run playerRun2 = runs2.stream().filter(r -> "Knocked".equals(r.text)).reduce((a, b) -> b).orElseThrow();
		int idx2 = runs2.indexOf(playerRun2);
		assertEquals(" stays unconscious.", runs2.get(idx2 + 1).text);
	}

	@Test
	public void heatExhaustionAndUnzappedPlayers() {
		// playerIdTouchdown is null in this test's report; without this stub the deep-stub
		// Game mock would auto-vivify a non-null Player mock for getPlayerById(null) and the
		// handler would spuriously render a touchdown block.
		given(game.getPlayerById((String) null)).willReturn(null);
		given(game.getPlayerById("he1")).willReturn(heated);
		given(heated.getName()).willReturn("Heated");
		given(game.getTeamHome().hasPlayer(heated)).willReturn(true);
		given(zapped.getName()).willReturn("Zapped");
		given(game.getTeamHome().hasPlayer(zapped)).willReturn(true);

		ReportTurnEnd report = new ReportTurnEnd(null, new KnockoutRecovery[0],
			new HeatExhaustion[] { new HeatExhaustion("he1", true, 3) },
			List.of(zapped), 9);
		List<Run> runs = render(new TurnEndMessage(), report);

		assertTrue(runs.stream().anyMatch(r -> "Heat Exhaustion Roll [ 9 ] ".equals(r.text) && r.textStyle == TextStyle.ROLL));
		Run heatedRun = runs.stream().filter(r -> "Heated".equals(r.text)).findFirst().orElseThrow();
		int heatedIdx = runs.indexOf(heatedRun);
		assertEquals(" is suffering from heat exhaustion.", runs.get(heatedIdx + 1).text);
		Run zappedRun = runs.stream().filter(r -> "Zapped".equals(r.text)).findFirst().orElseThrow();
		assertEquals(TextStyle.HOME_BOLD, zappedRun.textStyle);
		int zappedIdx = runs.indexOf(zappedRun);
		assertEquals(" recovers from Zap! spell effect.", runs.get(zappedIdx + 1).text);
	}

	@Test
	public void noRegularTurnLineWhenNotInRegularMode() {
		// playerIdTouchdown is null in this test's report; without this stub the deep-stub
		// Game mock would auto-vivify a non-null Player mock for getPlayerById(null), causing
		// the handler to spuriously render a touchdown block and break the isEmpty() assertion.
		given(game.getPlayerById((String) null)).willReturn(null);
		given(game.getTurnMode()).willReturn(TurnMode.KICKOFF);

		ReportTurnEnd report = new ReportTurnEnd(null, new KnockoutRecovery[0], new HeatExhaustion[0], Collections.emptyList(), 0);
		List<Run> runs = render(new TurnEndMessage(), report);

		assertTrue(runs.isEmpty());
	}
}
