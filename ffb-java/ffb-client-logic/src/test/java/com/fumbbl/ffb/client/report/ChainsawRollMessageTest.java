package com.fumbbl.ffb.client.report;

import com.fumbbl.ffb.PlayerGender;
import com.fumbbl.ffb.model.Player;
import com.fumbbl.ffb.report.ReportChainsawRoll;
import org.junit.jupiter.api.Test;
import org.mockito.Mock;

import java.util.List;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.BDDMockito.given;

class ChainsawRollMessageTest extends ReportMessageTestBase {

	@SuppressWarnings("rawtypes")
	@Mock
	private Player player;

	@SuppressWarnings("rawtypes")
	@Mock
	private Player defender;

	@Test
	public void successfulUsePrintsGenderGenitive() {
		given(game.getActingPlayer().getPlayer()).willReturn(player);
		given(player.getName()).willReturn("Chopper");
		given(player.getPlayerGender()).willReturn(PlayerGender.MALE);
		given(game.getTeamHome().hasPlayer(player)).willReturn(true);

		ReportChainsawRoll report = new ReportChainsawRoll("p1", true, 5, 2, false, null, null);
		List<Run> runs = render(new ChainsawRollMessage(), report);

		assertTrue(runs.stream().anyMatch(r -> " uses his Chainsaw".equals(r.text)));
	}

	@Test
	public void failedUsePrintsKickbackMessage() {
		given(game.getActingPlayer().getPlayer()).willReturn(player);
		given(player.getName()).willReturn("Chopper");
		given(player.getPlayerGender()).willReturn(PlayerGender.MALE);
		given(game.getTeamHome().hasPlayer(player)).willReturn(true);

		ReportChainsawRoll report = new ReportChainsawRoll("p1", false, 1, 2, false, null, null);
		List<Run> runs = render(new ChainsawRollMessage(), report);

		assertTrue(runs.stream().anyMatch(r -> "'s Chainsaw kicks back to hurt him".equals(r.text)));
	}

	@Test
	public void withDefenderPrintsAgainstDefender() {
		given(game.getActingPlayer().getPlayer()).willReturn(player);
		given(player.getName()).willReturn("Chopper");
		given(player.getPlayerGender()).willReturn(PlayerGender.MALE);
		given(game.getTeamHome().hasPlayer(player)).willReturn(true);
		given(game.getPlayerById("d1")).willReturn(defender);
		given(defender.getName()).willReturn("Target");

		ReportChainsawRoll report = new ReportChainsawRoll("p1", true, 6, 2, false, null, "d1");
		List<Run> runs = render(new ChainsawRollMessage(), report);

		assertTrue(runs.stream().anyMatch(r -> " against ".equals(r.text)));
		assertTrue(runs.stream().anyMatch(r -> "Target".equals(r.text)));
	}

	@Test
	public void withoutDefenderSkipsAgainstText() {
		given(game.getActingPlayer().getPlayer()).willReturn(player);
		given(player.getName()).willReturn("Chopper");
		given(player.getPlayerGender()).willReturn(PlayerGender.MALE);
		given(game.getTeamHome().hasPlayer(player)).willReturn(true);

		ReportChainsawRoll report = new ReportChainsawRoll("p1", true, 6, 2, false, null, null);
		List<Run> runs = render(new ChainsawRollMessage(), report);

		assertTrue(runs.stream().noneMatch(r -> " against ".equals(r.text)));
	}

	@Test
	public void reportIdIsChainsawRoll() {
		assertEquals("chainsawRoll", new ChainsawRollMessage().getKey());
	}
}
