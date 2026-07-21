package com.fumbbl.ffb.client.report;

import com.fumbbl.ffb.FactoryType;
import com.fumbbl.ffb.client.TextStyle;
import com.fumbbl.ffb.mechanics.Mechanic;
import com.fumbbl.ffb.mechanics.bb2025.AgilityMechanic;
import com.fumbbl.ffb.model.Player;
import com.fumbbl.ffb.report.ReportCatchRoll;
import org.junit.jupiter.api.Test;
import org.mockito.Mock;

import java.util.List;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.BDDMockito.given;

class CatchRollMessageTest extends ReportMessageTestBase {

	@SuppressWarnings("rawtypes")
	@Mock
	private Player player;

	private void stubMechanic() {
		given(game.getRules().getFactory(FactoryType.Factory.MECHANIC).forName(Mechanic.Type.AGILITY.name()))
			.willReturn(new AgilityMechanic());
	}

	@Test
	public void successfulCatchPrintsIntroAndSuccess() {
		given(game.getPlayerById("p1")).willReturn(player);
		given(player.getName()).willReturn("Catcher");
		given(game.getTeamHome().hasPlayer(player)).willReturn(true);
		stubMechanic();

		ReportCatchRoll report = new ReportCatchRoll("p1", true, 4, 3, false, null, false);
		List<Run> runs = render(new CatchRollMessage(), report);

		assertEquals("Catcher", runs.get(0).text);
		assertEquals(" tries to catch the ball:", runs.get(1).text);
		assertTrue(runs.stream().anyMatch(r -> "Catch Roll [ 4 ]".equals(r.text)));
		assertTrue(runs.stream().anyMatch(r -> " catches the ball.".equals(r.text)));
	}

	@Test
	public void failedCatchPrintsFailMessageAndNeededRoll() {
		given(game.getPlayerById("p1")).willReturn(player);
		given(player.getName()).willReturn("Catcher");
		given(game.getTeamHome().hasPlayer(player)).willReturn(true);
		stubMechanic();

		ReportCatchRoll report = new ReportCatchRoll("p1", false, 1, 3, false, null, false);
		List<Run> runs = render(new CatchRollMessage(), report);

		assertTrue(runs.stream().anyMatch(r -> " fails the catch.".equals(r.text)));
		Run needed = runs.stream().filter(r -> r.textStyle == TextStyle.NEEDED_ROLL).findFirst().orElseThrow();
		assertTrue(needed.text.startsWith("Roll a 3+ to succeed"));
	}

	@Test
	public void bombCatchUsesBombWording() {
		given(game.getPlayerById("p1")).willReturn(player);
		given(player.getName()).willReturn("Catcher");
		given(game.getTeamHome().hasPlayer(player)).willReturn(true);
		stubMechanic();

		ReportCatchRoll report = new ReportCatchRoll("p1", true, 5, 2, false, null, true);
		List<Run> runs = render(new CatchRollMessage(), report);

		assertEquals(" tries to catch the bomb:", runs.get(1).text);
		assertTrue(runs.stream().anyMatch(r -> " catches the bomb.".equals(r.text)));
	}

	@Test
	public void reRolledCatchSkipsIntroAndNeededRoll() {
		given(game.getPlayerById("p1")).willReturn(player);
		given(player.getName()).willReturn("Catcher");
		given(game.getTeamHome().hasPlayer(player)).willReturn(true);

		ReportCatchRoll report = new ReportCatchRoll("p1", true, 4, 3, true, null, false);
		List<Run> runs = render(new CatchRollMessage(), report);

		assertTrue(runs.stream().noneMatch(r -> " tries to catch the ball:".equals(r.text)));
		assertTrue(runs.stream().noneMatch(r -> r.textStyle == TextStyle.NEEDED_ROLL));
	}

	@Test
	public void reportIdIsCatchRoll() {
		assertEquals("catchRoll", new CatchRollMessage().getKey());
	}
}
