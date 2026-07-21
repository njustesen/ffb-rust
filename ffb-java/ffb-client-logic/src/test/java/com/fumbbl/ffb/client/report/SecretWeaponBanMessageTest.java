package com.fumbbl.ffb.client.report;

import com.fumbbl.ffb.model.Player;
import com.fumbbl.ffb.report.ReportSecretWeaponBan;
import org.junit.jupiter.api.Test;
import org.mockito.Mock;

import java.util.List;
import java.util.stream.Collectors;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.BDDMockito.given;

class SecretWeaponBanMessageTest extends ReportMessageTestBase {

	@SuppressWarnings("rawtypes")
	@Mock
	private Player player;

	@Test
	public void bannedPlayerRendersBanMessageAndPenalty() {
		given(game.getPlayerById("p1")).willReturn(player);
		given(player.getName()).willReturn("Ripper");
		given(game.getTeamHome().hasPlayer(player)).willReturn(true);

		ReportSecretWeaponBan report = new ReportSecretWeaponBan();
		report.add("p1", 3, true);
		List<String> texts = render(new SecretWeaponBanMessage(), report).stream().map(r -> r.text).collect(Collectors.toList());

		assertTrue(texts.contains("The ref bans "));
		assertTrue(texts.contains(" for using a Secret Weapon."));
		assertTrue(texts.stream().anyMatch(t -> t != null && t.startsWith("Penalty roll was 3")));
	}

	@Test
	public void overlookedPlayerRendersOverlookMessage() {
		given(game.getPlayerById("p1")).willReturn(player);
		given(player.getName()).willReturn("Ripper");
		given(game.getTeamHome().hasPlayer(player)).willReturn(true);

		ReportSecretWeaponBan report = new ReportSecretWeaponBan();
		report.add("p1", 0, false);
		List<String> texts = render(new SecretWeaponBanMessage(), report).stream().map(r -> r.text).collect(Collectors.toList());

		assertTrue(texts.contains("The ref overlooks "));
		assertTrue(texts.contains(" using a Secret Weapon."));
		assertFalse(texts.stream().anyMatch(t -> t != null && t.startsWith("Penalty roll was")));
	}

	@Test
	public void emptyPlayerIdsRendersNothing() {
		ReportSecretWeaponBan report = new ReportSecretWeaponBan();
		List<Run> runs = render(new SecretWeaponBanMessage(), report);

		assertTrue(runs.isEmpty());
	}

	@Test
	public void playerOnOtherTeamIsSkipped() {
		given(game.getPlayerById("p1")).willReturn(player);
		given(player.getName()).willReturn("Away Guy");
		given(game.getTeamHome().hasPlayer(player)).willReturn(false);
		given(game.getTeamAway().hasPlayer(player)).willReturn(true);

		ReportSecretWeaponBan report = new ReportSecretWeaponBan();
		report.add("p1", 3, true);
		List<String> texts = render(new SecretWeaponBanMessage(), report).stream().map(r -> r.text).collect(Collectors.toList());

		long banCount = texts.stream().filter(t -> "The ref bans ".equals(t)).count();
		assertEquals(1, banCount);
	}
}
