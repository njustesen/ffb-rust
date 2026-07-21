package com.fumbbl.ffb.client.report.bb2025;

import com.fumbbl.ffb.client.ParagraphStyle;
import com.fumbbl.ffb.client.TextStyle;
import com.fumbbl.ffb.client.report.ReportMessageTestBase;
import com.fumbbl.ffb.report.ReportId;
import com.fumbbl.ffb.report.mixed.ReportKickTeamMateFumble;
import org.junit.jupiter.api.Test;

import java.util.List;

import static org.junit.jupiter.api.Assertions.assertEquals;

class KickTeamMateFumbleMessageTest extends ReportMessageTestBase {

	@Test
	public void printsExplanationAtIndentPlusTwo() {
		statusReport.setIndent(1);
		ReportKickTeamMateFumble report = new ReportKickTeamMateFumble();
		List<Run> runs = render(new KickTeamMateFumbleMessage(), report);
		assertEquals("Fumbled Kick Team-Mate always causes at least a KO.", runs.get(0).text);
	}

	@Test
	public void usesExplanationStyle() {
		ReportKickTeamMateFumble report = new ReportKickTeamMateFumble();
		List<Run> runs = render(new KickTeamMateFumbleMessage(), report);
		assertEquals(TextStyle.EXPLANATION, runs.get(0).textStyle);
	}

	@Test
	public void indentIsBaseIndentPlusTwo() {
		statusReport.setIndent(0);
		ReportKickTeamMateFumble report = new ReportKickTeamMateFumble();
		List<Run> runs = render(new KickTeamMateFumbleMessage(), report);
		assertEquals(ParagraphStyle.INDENT_2, runs.get(0).paragraphStyle);
	}

	@Test
	public void reportIdIsKickTeamMateFumble() {
		assertEquals(ReportId.KICK_TEAM_MATE_FUMBLE.getKey(), new KickTeamMateFumbleMessage().getKey());
	}
}
