package com.fumbbl.ffb.client.report.bb2025;

import com.fumbbl.ffb.PlayerGender;
import com.fumbbl.ffb.client.TextStyle;
import com.fumbbl.ffb.client.report.ReportMessageTestBase;
import com.fumbbl.ffb.model.Player;
import com.fumbbl.ffb.report.bb2025.ReportThrowAtPlayer;
import org.junit.jupiter.api.Test;
import org.mockito.Mock;

import java.util.List;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.BDDMockito.given;

class ThrowAtPlayerMessageTest extends ReportMessageTestBase {

	@SuppressWarnings("rawtypes")
	@Mock
	private Player victim;

	@SuppressWarnings("rawtypes")
	@Mock
	private Player target;

	@Test
	public void getKeyIsThrowAtPlayer() {
		assertEquals("throwAtPlayer", new ThrowAtPlayerMessage().getKey());
	}

	@Test
	public void successfulHitKnocksHimDown() {
		given(game.getPlayerById("p1")).willReturn(victim);
		given(victim.getName()).willReturn("Victim");
		given(victim.getPlayerGender()).willReturn(PlayerGender.MALE);
		given(game.getTeamHome().hasPlayer(victim)).willReturn(true);

		ReportThrowAtPlayer report = new ReportThrowAtPlayer("p1", 6, true);
		List<Run> runs = render(new ThrowAtPlayerMessage(), report);

		assertTrue(runs.stream().anyMatch(r -> "Throw a Rock Roll [ 6 ]".equals(r.text)));
		assertTrue(runs.stream().anyMatch(r -> " knocking him down.".equals(r.text)));
	}

	@Test
	public void missUsesDativePronoun() {
		given(game.getPlayerById("p2")).willReturn(target);
		given(target.getName()).willReturn("Target");
		given(target.getPlayerGender()).willReturn(PlayerGender.FEMALE);
		given(game.getTeamHome().hasPlayer(target)).willReturn(false);

		ReportThrowAtPlayer report = new ReportThrowAtPlayer("p2", 1, false);
		List<Run> runs = render(new ThrowAtPlayerMessage(), report);

		assertTrue(runs.stream().anyMatch(r -> " but miss her.".equals(r.text)));
	}

	@Test
	public void printsPlayerBold() {
		given(game.getPlayerById("p1")).willReturn(victim);
		given(victim.getName()).willReturn("Victim");
		given(victim.getPlayerGender()).willReturn(PlayerGender.MALE);
		given(game.getTeamHome().hasPlayer(victim)).willReturn(true);

		ReportThrowAtPlayer report = new ReportThrowAtPlayer("p1", 4, true);
		List<Run> runs = render(new ThrowAtPlayerMessage(), report);

		Run run = runs.stream().filter(r -> "Victim".equals(r.text)).findFirst().orElseThrow();
		assertEquals(TextStyle.HOME_BOLD, run.textStyle);
	}
}
