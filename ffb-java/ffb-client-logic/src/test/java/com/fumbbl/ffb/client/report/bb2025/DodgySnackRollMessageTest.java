package com.fumbbl.ffb.client.report.bb2025;

import com.fumbbl.ffb.client.report.ReportMessageTestBase;
import com.fumbbl.ffb.model.Player;
import com.fumbbl.ffb.report.ReportId;
import com.fumbbl.ffb.report.bb2025.ReportDodgySnackRoll;
import org.junit.jupiter.api.Test;
import org.mockito.Mock;

import java.util.List;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.BDDMockito.given;

class DodgySnackRollMessageTest extends ReportMessageTestBase {

	@SuppressWarnings("rawtypes")
	@Mock
	private Player player;

	@Test
	public void rollOfOneSendsToReserves() {
		given(game.getPlayerById("p1")).willReturn(player);
		given(game.getTeamHome().hasPlayer(player)).willReturn(true);

		ReportDodgySnackRoll report = new ReportDodgySnackRoll(1, "p1");
		List<Run> runs = render(new DodgySnackRollMessage(), report);
		List<String> texts = runs.stream().map(r -> r.text).collect(java.util.stream.Collectors.toList());
		assertTrue(texts.stream().anyMatch(t -> " is sent to reserves.".equals(t)));
	}

	@Test
	public void otherRollSuffersMaAvPenalty() {
		given(game.getPlayerById("p1")).willReturn(player);
		given(game.getTeamHome().hasPlayer(player)).willReturn(true);

		ReportDodgySnackRoll report = new ReportDodgySnackRoll(4, "p1");
		List<Run> runs = render(new DodgySnackRollMessage(), report);
		List<String> texts = runs.stream().map(r -> r.text).collect(java.util.stream.Collectors.toList());
		assertTrue(texts.stream().anyMatch(t -> " suffers -MA and -AV for this drive.".equals(t)));
	}

	@Test
	public void printsRollHeader() {
		given(game.getPlayerById("p1")).willReturn(player);
		given(game.getTeamHome().hasPlayer(player)).willReturn(true);
		given(player.getName()).willReturn("Player One");

		ReportDodgySnackRoll report = new ReportDodgySnackRoll(6, "p1");
		List<Run> runs = render(new DodgySnackRollMessage(), report);
		List<String> texts = runs.stream().map(r -> r.text).collect(java.util.stream.Collectors.toList());
		assertTrue(texts.stream().anyMatch(t -> "Dodgy Snack Effect Roll [ 6 ]".equals(t)));
		assertTrue(texts.stream().anyMatch(t -> "Player One".equals(t)));
	}

	@Test
	public void reportIdIsDodgySnackRoll() {
		assertEquals(ReportId.DODGY_SNACK_ROLL.getKey(), new DodgySnackRollMessage().getKey());
	}
}
