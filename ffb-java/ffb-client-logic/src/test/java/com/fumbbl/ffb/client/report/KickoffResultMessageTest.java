package com.fumbbl.ffb.client.report;

import com.fumbbl.ffb.kickoff.bb2020.KickoffResult;
import com.fumbbl.ffb.report.ReportKickoffResult;
import org.junit.jupiter.api.Test;

import java.util.List;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;

class KickoffResultMessageTest extends ReportMessageTestBase {

	@Test
	public void rolledEventPrintsRollValues() {
		ReportKickoffResult report = new ReportKickoffResult(KickoffResult.BLITZ, new int[]{3, 4});
		List<Run> runs = render(new KickoffResultMessage(), report);

		assertTrue(runs.stream().anyMatch(r -> "Kick-off Event Roll [ 3 ][ 4 ]".equals(r.text)));
		assertTrue(runs.stream().anyMatch(r -> "Kick-off event is Blitz".equals(r.text)));
	}

	@Test
	public void chosenEventHasNoRoll() {
		ReportKickoffResult report = new ReportKickoffResult(KickoffResult.QUICK_SNAP, new int[0]);
		List<Run> runs = render(new KickoffResultMessage(), report);

		assertTrue(runs.stream().anyMatch(r -> "Chosen kick-off event".equals(r.text)));
	}

	@Test
	public void indentIsLeftAt1AfterRender() {
		ReportKickoffResult report = new ReportKickoffResult(KickoffResult.BLITZ, new int[]{1, 2});
		render(new KickoffResultMessage(), report);

		assertEquals(1, statusReport.getIndent());
	}
}
