package com.fumbbl.ffb.client.report.bb2016;

import com.fumbbl.ffb.client.TextStyle;
import com.fumbbl.ffb.client.report.ReportMessageTestBase;
import com.fumbbl.ffb.model.Player;
import com.fumbbl.ffb.modifiers.RollModifier;
import com.fumbbl.ffb.report.ReportGoForItRoll;
import org.junit.jupiter.api.Test;
import org.mockito.Mock;

import java.util.List;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.mockito.BDDMockito.given;

class GoForItRollMessageTest extends ReportMessageTestBase {

	@SuppressWarnings("rawtypes")
	@Mock
	private Player thrower;

	@Test
	public void getKeyIsGoForItRoll() {
		assertEquals("goForItRoll", new GoForItRollMessage().getKey());
	}

	@Test
	public void successfulRollReportsGoesForItAndNeededRoll() {
		given(game.getActingPlayer().getPlayer()).willReturn(thrower);
		given(game.getTeamHome().hasPlayer(thrower)).willReturn(true);
		given(thrower.getName()).willReturn("Grubb");

		ReportGoForItRoll report = new ReportGoForItRoll("p1", true, 3, 2, false, new RollModifier<?>[0]);
		List<Run> runs = render(new GoForItRollMessage(), report);

		assertEquals(" goes for it!", runs.get(3).text);
		Run needed = runs.stream().filter(r -> r.textStyle == TextStyle.NEEDED_ROLL).findFirst().orElseThrow();
		assertEquals("Succeeded on a roll of 2+ (Roll > 1).", needed.text);
	}

	@Test
	public void failedRollTrips() {
		given(game.getActingPlayer().getPlayer()).willReturn(thrower);
		given(game.getTeamHome().hasPlayer(thrower)).willReturn(true);
		given(thrower.getName()).willReturn("Grubb");

		ReportGoForItRoll report = new ReportGoForItRoll("p1", false, 1, 2, false, new RollModifier<?>[0]);
		List<Run> runs = render(new GoForItRollMessage(), report);

		assertEquals(" trips while going for it.", runs.get(3).text);
	}

	@Test
	public void reRolledHasNoNeededRollLine() {
		given(game.getActingPlayer().getPlayer()).willReturn(thrower);
		given(game.getTeamHome().hasPlayer(thrower)).willReturn(true);
		given(thrower.getName()).willReturn("Grubb");

		ReportGoForItRoll report = new ReportGoForItRoll("p1", false, 1, 2, true, new RollModifier<?>[0]);
		List<Run> runs = render(new GoForItRollMessage(), report);

		assertFalse(runs.stream().anyMatch(r -> r.textStyle == TextStyle.NEEDED_ROLL));
	}
}
