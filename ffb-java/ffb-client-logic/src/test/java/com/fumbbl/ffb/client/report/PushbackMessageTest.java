package com.fumbbl.ffb.client.report;

import com.fumbbl.ffb.PlayerGender;
import com.fumbbl.ffb.PushbackMode;
import com.fumbbl.ffb.model.Player;
import com.fumbbl.ffb.report.ReportPushback;
import org.junit.jupiter.api.Test;
import org.mockito.Mock;

import java.util.List;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.BDDMockito.given;

class PushbackMessageTest extends ReportMessageTestBase {

	@SuppressWarnings("rawtypes")
	@Mock
	private Player defender;

	@SuppressWarnings("rawtypes")
	@Mock
	private Player actor;

	@Test
	public void renderSideStepPrintsDefenderMessage() {
		given(game.getPlayerById("d1")).willReturn(defender);
		given(defender.getName()).willReturn("Defender");
		given(game.getTeamHome().hasPlayer(defender)).willReturn(false);

		ReportPushback report = new ReportPushback("d1", PushbackMode.SIDE_STEP);
		List<Run> runs = render(new PushbackMessage(), report);

		assertEquals("Defender", runs.get(0).text);
		assertEquals(" uses Side Step to avoid being pushed.", runs.get(1).text);
	}

	@Test
	public void renderGrabUsesActingPlayerGender() {
		given(game.getActingPlayer().getPlayer()).willReturn(actor);
		given(actor.getName()).willReturn("Actor");
		given(actor.getPlayerGender()).willReturn(PlayerGender.FEMALE);
		given(game.getTeamHome().hasPlayer(actor)).willReturn(true);

		ReportPushback report = new ReportPushback("d1", PushbackMode.GRAB);
		List<Run> runs = render(new PushbackMessage(), report);

		assertEquals("Actor", runs.get(0).text);
		assertEquals(" uses Grab to place her opponent.", runs.get(1).text);
	}

	@Test
	public void renderRegularModePrintsNothing() {
		ReportPushback report = new ReportPushback("d1", PushbackMode.REGULAR);
		List<Run> runs = render(new PushbackMessage(), report);

		assertTrue(runs.isEmpty());
	}
}
