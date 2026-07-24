package com.fumbbl.ffb.client.report;

import com.fumbbl.ffb.PlayerGender;
import com.fumbbl.ffb.model.Player;
import com.fumbbl.ffb.report.ReportStandUpRoll;
import org.junit.jupiter.api.Test;
import org.mockito.Mock;

import java.util.List;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.BDDMockito.given;

class StandUpRollMessageTest extends ReportMessageTestBase {

	@SuppressWarnings("rawtypes")
	@Mock
	private Player player;

	private void givenActingPlayer() {
		given(game.getActingPlayer().getPlayer()).willReturn(player);
		given(player.getName()).willReturn("Thorsson");
		given(player.getPlayerGender()).willReturn(PlayerGender.MALE);
		given(game.getTeamHome().hasPlayer(player)).willReturn(true);
	}

	@Test
	public void successfulStandUpReportsSuccessAndNeededRollWhenNotRerolled() {
		givenActingPlayer();

		ReportStandUpRoll report = new ReportStandUpRoll("p1", true, 4, 1, false);
		List<Run> runs = render(new StandUpRollMessage(), report);

		assertTrue(runs.stream().anyMatch(r -> "Stand Up Roll [ 4 ]".equals(r.text)));
		assertTrue(runs.stream().anyMatch(r -> " stands up.".equals(r.text)));
		assertTrue(runs.stream().anyMatch(r -> "Succeeded on a roll of 3+.".equals(r.text)));
	}

	@Test
	public void failedStandUpReportsFailureAndNeededRoll() {
		givenActingPlayer();

		ReportStandUpRoll report = new ReportStandUpRoll("p1", false, 1, 1, false);
		List<Run> runs = render(new StandUpRollMessage(), report);

		assertTrue(runs.stream().anyMatch(r -> " doesn't get to his feet.".equals(r.text)));
		assertTrue(runs.stream().anyMatch(r -> "Roll a 3+ to succeed.".equals(r.text)));
	}

	@Test
	public void rerolledSuccessOmitsNeededRoll() {
		givenActingPlayer();

		ReportStandUpRoll report = new ReportStandUpRoll("p1", true, 4, 1, true);
		List<Run> runs = render(new StandUpRollMessage(), report);

		assertFalse(runs.stream().anyMatch(r -> r.text != null && r.text.startsWith("Succeeded on a roll of")));
	}

	@Test
	public void rerolledFailureOmitsNeededRoll() {
		givenActingPlayer();

		ReportStandUpRoll report = new ReportStandUpRoll("p1", false, 1, 1, true);
		List<Run> runs = render(new StandUpRollMessage(), report);

		assertFalse(runs.stream().anyMatch(r -> r.text != null && r.text.startsWith("Roll a")));
	}

}
