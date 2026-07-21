package com.fumbbl.ffb.client.report.bb2025;

import com.fumbbl.ffb.client.report.ReportMessageTestBase;
import com.fumbbl.ffb.model.Player;
import com.fumbbl.ffb.report.ReportId;
import com.fumbbl.ffb.report.bb2025.ReportKickoffDodgySnack;
import org.junit.jupiter.api.Test;
import org.mockito.Mock;

import java.util.Collections;
import java.util.List;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.BDDMockito.given;

class KickoffDodgySnackMessageTest extends ReportMessageTestBase {

	@SuppressWarnings("rawtypes")
	@Mock
	private Player playerHome;

	@SuppressWarnings("rawtypes")
	@Mock
	private Player playerAway;

	@Test
	public void reportsBothTeamRolls() {
		ReportKickoffDodgySnack report = new ReportKickoffDodgySnack(3, 5, Collections.emptyList());
		List<Run> runs = render(new KickoffDodgySnackMessage(), report);
		List<String> texts = runs.stream().map(r -> r.text).collect(java.util.stream.Collectors.toList());
		assertTrue(texts.contains("Dodgy Snack Roll Home Team [ 3 ]"));
		assertTrue(texts.contains("Dodgy Snack Roll Away Team [ 5 ]"));
	}

	@Test
	public void noPlayersMeansNoSnackLines() {
		ReportKickoffDodgySnack report = new ReportKickoffDodgySnack(1, 1, Collections.emptyList());
		List<Run> runs = render(new KickoffDodgySnackMessage(), report);
		List<String> texts = runs.stream().map(r -> r.text).collect(java.util.stream.Collectors.toList());
		assertFalse(texts.stream().anyMatch(t -> t != null && t.contains("dodgy snack")));
	}

	@Test
	public void eachPlayerGetsADodgySnackLine() {
		given(game.getPlayerById("p1")).willReturn(playerHome);
		given(playerHome.getName()).willReturn("Hungry Guy");
		given(game.getTeamHome().hasPlayer(playerHome)).willReturn(true);

		given(game.getPlayerById("p2")).willReturn(playerAway);
		given(playerAway.getName()).willReturn("Snacker");
		given(game.getTeamHome().hasPlayer(playerAway)).willReturn(false);

		ReportKickoffDodgySnack report = new ReportKickoffDodgySnack(2, 2, java.util.Arrays.asList("p1", "p2"));
		List<Run> runs = render(new KickoffDodgySnackMessage(), report);
		List<String> texts = runs.stream().map(r -> r.text).collect(java.util.stream.Collectors.toList());
		assertTrue(texts.stream().anyMatch(t -> "Hungry Guy".equals(t)));
		assertTrue(texts.stream().anyMatch(t -> "Snacker".equals(t)));
		assertEquals(2, texts.stream().filter(t -> " had a dodgy snack.".equals(t)).count());
	}

	@Test
	public void reportIdIsKickoffDodgySnack() {
		assertEquals(ReportId.KICKOFF_DODGY_SNACK.getKey(), new KickoffDodgySnackMessage().getKey());
	}
}
