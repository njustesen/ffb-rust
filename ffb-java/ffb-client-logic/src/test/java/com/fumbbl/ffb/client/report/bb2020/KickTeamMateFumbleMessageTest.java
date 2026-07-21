package com.fumbbl.ffb.client.report.bb2020;

import com.fumbbl.ffb.client.TextStyle;
import com.fumbbl.ffb.client.report.ReportMessageTestBase;
import com.fumbbl.ffb.report.mixed.ReportKickTeamMateFumble;
import org.junit.jupiter.api.Test;

import java.util.List;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;

class KickTeamMateFumbleMessageTest extends ReportMessageTestBase {

	@Test
	public void rendersExplanationText() {
		ReportKickTeamMateFumble report = new ReportKickTeamMateFumble();
		List<Run> runs = render(new KickTeamMateFumbleMessage(), report);
		assertTrue(runs.stream().anyMatch(r ->
			"Fumbled Kick Team-Mate always removes kicked player and causes at least a KO.".equals(r.text)));
	}

	@Test
	public void usesExplanationTextStyle() {
		ReportKickTeamMateFumble report = new ReportKickTeamMateFumble();
		List<Run> runs = render(new KickTeamMateFumbleMessage(), report);
		assertEquals(TextStyle.EXPLANATION, runs.get(0).textStyle);
	}
}
