package com.fumbbl.ffb.client.report;

import com.fumbbl.ffb.PlayerGender;
import com.fumbbl.ffb.model.Player;
import com.fumbbl.ffb.report.ReportDauntlessRoll;
import org.junit.jupiter.api.Test;
import org.mockito.Mock;

import java.util.List;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.BDDMockito.given;

class DauntlessRollMessageTest extends ReportMessageTestBase {

	@SuppressWarnings("rawtypes")
	@Mock
	private Player player;

	@SuppressWarnings("rawtypes")
	@Mock
	private Player defender;

	private void givenActingPlayer() {
		given(game.getActingPlayer().getPlayer()).willReturn(player);
		given(player.getName()).willReturn("Strongman");
		given(player.getPlayerGender()).willReturn(PlayerGender.FEMALE);
		given(game.getTeamHome().hasPlayer(player)).willReturn(true);
	}

	@Test
	public void successfulPushReportsNewStrength() {
		givenActingPlayer();

		ReportDauntlessRoll report = new ReportDauntlessRoll("p1", true, 5, 3, false, 5, null);
		List<Run> runs = render(new DauntlessRollMessage(), report);

		assertTrue(runs.stream().anyMatch(r -> " uses Dauntless to push herself to strength 5".equals(r.text)));
	}

	@Test
	public void failedPushReportsGenitiveStrength() {
		givenActingPlayer();

		ReportDauntlessRoll report = new ReportDauntlessRoll("p1", false, 1, 3, false, 3, null);
		List<Run> runs = render(new DauntlessRollMessage(), report);

		assertTrue(runs.stream().anyMatch(r -> " fails to push her strength".equals(r.text)));
	}

	@Test
	public void withDefenderPrintsToMatchDefender() {
		givenActingPlayer();
		given(game.getPlayerById("d1")).willReturn(defender);
		given(defender.getName()).willReturn("Blocker");
		given(game.getTeamHome().hasPlayer(defender)).willReturn(false);

		ReportDauntlessRoll report = new ReportDauntlessRoll("p1", true, 6, 3, false, 5, "d1");
		List<Run> runs = render(new DauntlessRollMessage(), report);

		assertTrue(runs.stream().anyMatch(r -> " to match ".equals(r.text)));
		assertTrue(runs.stream().anyMatch(r -> "Blocker".equals(r.text)));
	}

	@Test
	public void withoutDefenderSkipsToMatchText() {
		givenActingPlayer();

		ReportDauntlessRoll report = new ReportDauntlessRoll("p1", true, 6, 3, false, 5, null);
		List<Run> runs = render(new DauntlessRollMessage(), report);

		assertTrue(runs.stream().noneMatch(r -> " to match ".equals(r.text)));
	}

	@Test
	public void reportIdIsDauntlessRoll() {
		assertEquals("dauntlessRoll", new DauntlessRollMessage().getKey());
	}
}
