package com.fumbbl.ffb.client.report.mixed;

import com.fumbbl.ffb.client.TextStyle;
import com.fumbbl.ffb.client.report.ReportMessageTestBase;
import com.fumbbl.ffb.report.mixed.ReportKickoffSequenceActivationsCount;
import org.junit.jupiter.api.Test;

import java.util.List;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;

class KickoffSequenceActivationsCountMessageTest extends ReportMessageTestBase {

	@Test
	public void rendersPluralRemainText() {
		ReportKickoffSequenceActivationsCount report = new ReportKickoffSequenceActivationsCount(5, 2, 3);
		List<Run> runs = render(new KickoffSequenceActivationsCountMessage(), report);

		assertEquals("Max 3 open players can be used - 2 used (5 remain open).", runs.get(0).text);
		assertEquals(TextStyle.EXPLANATION, runs.get(0).textStyle);
	}

	@Test
	public void rendersSingularRemainsTextWhenAvailableIsOne() {
		ReportKickoffSequenceActivationsCount report = new ReportKickoffSequenceActivationsCount(1, 4, 5);
		List<Run> runs = render(new KickoffSequenceActivationsCountMessage(), report);

		assertEquals("Max 5 open players can be used - 4 used (1 remains open).", runs.get(0).text);
	}

	@Test
	public void rendersAtCurrentIndentPlusOne() {
		statusReport.setIndent(1);
		ReportKickoffSequenceActivationsCount report = new ReportKickoffSequenceActivationsCount(3, 0, 3);
		List<Run> runs = render(new KickoffSequenceActivationsCountMessage(), report);

		assertTrue(runs.get(0).text.contains("Max 3"));
	}
}
