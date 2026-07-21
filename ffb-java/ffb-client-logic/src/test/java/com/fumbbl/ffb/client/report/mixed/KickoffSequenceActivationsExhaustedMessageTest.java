package com.fumbbl.ffb.client.report.mixed;

import com.fumbbl.ffb.client.TextStyle;
import com.fumbbl.ffb.client.report.ReportMessageTestBase;
import com.fumbbl.ffb.report.mixed.ReportKickoffSequenceActivationsExhausted;
import org.junit.jupiter.api.Test;

import java.util.List;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNotNull;

class KickoffSequenceActivationsExhaustedMessageTest extends ReportMessageTestBase {

	@Test
	public void rendersLimitReachedMessage() {
		ReportKickoffSequenceActivationsExhausted report = new ReportKickoffSequenceActivationsExhausted(true);
		List<Run> runs = render(new KickoffSequenceActivationsExhaustedMessage(), report);

		assertEquals("Moved allowed number of players.", runs.get(0).text);
		assertEquals(TextStyle.EXPLANATION, runs.get(0).textStyle);
	}

	@Test
	public void rendersNoMorePlayersMessage() {
		ReportKickoffSequenceActivationsExhausted report = new ReportKickoffSequenceActivationsExhausted(false);
		List<Run> runs = render(new KickoffSequenceActivationsExhaustedMessage(), report);

		assertEquals("No more open players available.", runs.get(0).text);
	}

	@Test
	public void rendersAtCurrentIndentPlusOne() {
		statusReport.setIndent(2);
		ReportKickoffSequenceActivationsExhausted report = new ReportKickoffSequenceActivationsExhausted(true);
		List<Run> runs = render(new KickoffSequenceActivationsExhaustedMessage(), report);

		assertNotNull(runs.get(0).text);
	}
}
