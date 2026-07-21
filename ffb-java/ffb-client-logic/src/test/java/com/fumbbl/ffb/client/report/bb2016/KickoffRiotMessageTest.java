package com.fumbbl.ffb.client.report.bb2016;

import com.fumbbl.ffb.client.report.ReportMessageTestBase;
import com.fumbbl.ffb.report.bb2016.ReportKickoffRiot;
import org.junit.jupiter.api.Test;

import java.util.List;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;

class KickoffRiotMessageTest extends ReportMessageTestBase {

	@Test
	public void getKeyIsKickoffRiot() {
		assertEquals("kickoffRiot", new KickoffRiotMessage().getKey());
	}

	@Test
	public void positiveRollReportsRollValue() {
		ReportKickoffRiot report = new ReportKickoffRiot(4, -2);
		List<Run> runs = render(new KickoffRiotMessage(), report);

		assertEquals("Riot Roll [ 4 ]", runs.get(0).text);
		assertTrue(runs.stream().anyMatch(r -> "Turn Counter is moved 2 steps backward.".equals(r.text)));
	}

	@Test
	public void zeroRollReportsTurnNumber() {
		ReportKickoffRiot report = new ReportKickoffRiot(0, 1);
		List<Run> runs = render(new KickoffRiotMessage(), report);

		assertTrue(runs.get(0).text.startsWith("Riot in Turn"));
	}

	@Test
	public void positiveModifierMovesForward() {
		ReportKickoffRiot report = new ReportKickoffRiot(3, 1);
		List<Run> runs = render(new KickoffRiotMessage(), report);

		assertTrue(runs.stream().anyMatch(r -> "Turn Counter is moved 1 steps forward.".equals(r.text)));
	}
}
