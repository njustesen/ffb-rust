package com.fumbbl.ffb.client.report.bb2016;

import com.fumbbl.ffb.PlayerGender;
import com.fumbbl.ffb.client.report.ReportMessageTestBase;
import com.fumbbl.ffb.model.Player;
import com.fumbbl.ffb.report.bb2016.ReportKickTeamMateRoll;
import org.junit.jupiter.api.Test;
import org.mockito.Mock;

import java.util.List;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.BDDMockito.given;

class KickTeamMateRollMessageTest extends ReportMessageTestBase {

	@SuppressWarnings("rawtypes")
	@Mock
	private Player kicker;

	@Test
	public void getKeyIsKickTeamMateRoll() {
		assertEquals("kickTeamMateRoll", new KickTeamMateRollMessage().getKey());
	}

	@Test
	public void successfulKickReportsSuccess() {
		given(game.getActingPlayer().getPlayer()).willReturn(kicker);
		given(kicker.getPlayerGender()).willReturn(PlayerGender.MALE);

		ReportKickTeamMateRoll report = new ReportKickTeamMateRoll("kicker", "kicked", true, new int[]{5}, false, 3);
		List<Run> runs = render(new KickTeamMateRollMessage(), report);

		assertTrue(runs.stream().anyMatch(r -> " kicks his team-mate successfully.".equals(r.text)));
	}

	@Test
	public void failedKickReportsTooEnthusiastic() {
		given(game.getActingPlayer().getPlayer()).willReturn(kicker);

		ReportKickTeamMateRoll report = new ReportKickTeamMateRoll("kicker", "kicked", false, new int[]{1}, false, 0);
		List<Run> runs = render(new KickTeamMateRollMessage(), report);

		assertTrue(runs.stream().anyMatch(r -> " is a bit too enthusiastic.".equals(r.text)));
	}

	@Test
	public void reRolledSkipsIntroLine() {
		given(game.getActingPlayer().getPlayer()).willReturn(kicker);
		given(kicker.getPlayerGender()).willReturn(PlayerGender.MALE);

		ReportKickTeamMateRoll report = new ReportKickTeamMateRoll("kicker", "kicked", true, new int[]{3, 4}, true, 3);
		List<Run> runs = render(new KickTeamMateRollMessage(), report);

		assertEquals("Kick Team-Mate Roll [ 3 ][ 4 ]", runs.get(0).text);
	}
}
