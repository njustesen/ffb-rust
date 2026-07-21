package com.fumbbl.ffb.client.report.mixed;

import com.fumbbl.ffb.PlayerGender;
import com.fumbbl.ffb.client.report.ReportMessageTestBase;
import com.fumbbl.ffb.model.Player;
import com.fumbbl.ffb.report.mixed.ReportLookIntoMyEyesRoll;
import org.junit.jupiter.api.Test;
import org.mockito.Mock;

import java.util.List;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.BDDMockito.given;

class LookIntoMyEyesRollMessageTest extends ReportMessageTestBase {

	@SuppressWarnings("rawtypes")
	@Mock
	private Player thrower;

	@Test
	public void successfulRoll() {
		given(game.getActingPlayer().getPlayer()).willReturn(thrower);
		given(thrower.getName()).willReturn("Grobnik");
		given(thrower.getPlayerGender()).willReturn(PlayerGender.MALE);
		given(game.getTeamHome().hasPlayer(thrower)).willReturn(true);

		ReportLookIntoMyEyesRoll report = new ReportLookIntoMyEyesRoll("p1", true, 4, false);
		List<Run> runs = render(new LookIntoMyEyesRollMessage(), report);

		assertEquals("Look Into My Eyes Roll [ 4 ]", runs.get(0).text);
		assertEquals("Grobnik", runs.get(2).text);
		assertTrue(runs.get(3).text.contains("steals the ball"));
	}

	@Test
	public void failedRoll() {
		given(game.getActingPlayer().getPlayer()).willReturn(thrower);
		given(thrower.getName()).willReturn("Grobnik");
		given(thrower.getPlayerGender()).willReturn(PlayerGender.MALE);
		given(game.getTeamHome().hasPlayer(thrower)).willReturn(true);

		ReportLookIntoMyEyesRoll report = new ReportLookIntoMyEyesRoll("p1", false, 1, false);
		List<Run> runs = render(new LookIntoMyEyesRollMessage(), report);

		assertTrue(runs.get(3).text.contains("fails to steal"));
	}
}
