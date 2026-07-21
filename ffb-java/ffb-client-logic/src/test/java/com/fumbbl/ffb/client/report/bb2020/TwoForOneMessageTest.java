package com.fumbbl.ffb.client.report.bb2020;

import com.fumbbl.ffb.client.report.ReportMessageTestBase;
import com.fumbbl.ffb.model.Player;
import com.fumbbl.ffb.report.bb2020.ReportTwoForOne;
import org.junit.jupiter.api.Test;
import org.mockito.Mock;

import java.util.List;

import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.BDDMockito.given;

class TwoForOneMessageTest extends ReportMessageTestBase {

	@SuppressWarnings("rawtypes")
	@Mock
	private Player p1;

	@SuppressWarnings("rawtypes")
	@Mock
	private Player p2;

	private void stubPlayers() {
		given(game.getPlayerById("p1")).willReturn(p1);
		given(game.getPlayerById("p2")).willReturn(p2);
		given(p1.getName()).willReturn("Player p1");
		given(p2.getName()).willReturn("Player p2");
	}

	@Test
	public void usedGainsLonerBecausePartnerInjured() {
		stubPlayers();
		ReportTwoForOne report = new ReportTwoForOne("p1", "p2", true);
		List<Run> runs = render(new TwoForOneMessage(), report);
		assertTrue(runs.stream().anyMatch(r -> r.text != null && r.text.contains(" gains Loner (2+) because ")));
		assertTrue(runs.stream().anyMatch(r -> r.text != null && r.text.contains(" is injured.")));
	}

	@Test
	public void notUsedLosesLonerBecausePartnerRecovered() {
		stubPlayers();
		ReportTwoForOne report = new ReportTwoForOne("p1", "p2", false);
		List<Run> runs = render(new TwoForOneMessage(), report);
		assertTrue(runs.stream().anyMatch(r -> r.text != null && r.text.contains(" loses Loner (2+) because ")));
		assertTrue(runs.stream().anyMatch(r -> r.text != null && r.text.contains(" has recovered.")));
	}

	@Test
	public void printsBothPlayerNames() {
		stubPlayers();
		ReportTwoForOne report = new ReportTwoForOne("p1", "p2", true);
		List<Run> runs = render(new TwoForOneMessage(), report);
		assertTrue(runs.stream().anyMatch(r -> "Player p1".equals(r.text)));
		assertTrue(runs.stream().anyMatch(r -> "Player p2".equals(r.text)));
	}
}
