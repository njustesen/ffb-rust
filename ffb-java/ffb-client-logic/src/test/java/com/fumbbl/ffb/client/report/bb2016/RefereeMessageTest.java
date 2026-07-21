package com.fumbbl.ffb.client.report.bb2016;

import com.fumbbl.ffb.client.report.ReportMessageTestBase;
import com.fumbbl.ffb.model.Player;
import com.fumbbl.ffb.report.bb2016.ReportReferee;
import org.junit.jupiter.api.Test;
import org.mockito.Mock;

import java.util.List;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.BDDMockito.given;

class RefereeMessageTest extends ReportMessageTestBase {

	@SuppressWarnings("rawtypes")
	@Mock
	private Player player;

	@Test
	public void getKeyIsReferee() {
		assertEquals("referee", new RefereeMessage().getKey());
	}

	@Test
	public void bannedReportsBanMessage() {
		given(game.getActingPlayer().getPlayer()).willReturn(player);
		given(player.getName()).willReturn("Grubb");
		given(game.getTeamHome().hasPlayer(player)).willReturn(true);

		ReportReferee report = new ReportReferee(true);
		List<Run> runs = render(new RefereeMessage(), report);

		assertTrue(runs.stream().anyMatch(r -> "Grubb".equals(r.text)));
		assertTrue(runs.stream().anyMatch(r -> " from the game.".equals(r.text)));
	}

	@Test
	public void notSpottedReportsDidntSpot() {
		ReportReferee report = new ReportReferee(false);
		List<Run> runs = render(new RefereeMessage(), report);

		assertTrue(runs.stream().anyMatch(r -> "The referee didn't spot the foul.".equals(r.text)));
	}
}
