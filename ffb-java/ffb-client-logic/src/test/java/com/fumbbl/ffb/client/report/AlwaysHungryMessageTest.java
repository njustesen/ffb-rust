package com.fumbbl.ffb.client.report;

import com.fumbbl.ffb.PlayerGender;
import com.fumbbl.ffb.client.TextStyle;
import com.fumbbl.ffb.model.Player;
import com.fumbbl.ffb.report.ReportAlwaysHungryRoll;
import org.junit.jupiter.api.Test;
import org.mockito.Mock;

import java.util.List;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.mockito.BDDMockito.given;

class AlwaysHungryMessageTest extends ReportMessageTestBase {

	@SuppressWarnings("rawtypes")
	@Mock
	private Player thrower;

	@Test
	public void rendersSuccess() {
		given(game.getActingPlayer().getPlayer()).willReturn(thrower);
		given(thrower.getName()).willReturn("Thrower");
		given(game.getTeamHome().hasPlayer(thrower)).willReturn(true);

		ReportAlwaysHungryRoll report = new ReportAlwaysHungryRoll("p1", true, 4, 2, false, null);
		List<Run> runs = render(new AlwaysHungryMessage(), report);

		assertEquals("Always Hungry Roll [ 4 ]", runs.get(0).text);
		assertEquals(TextStyle.ROLL, runs.get(0).textStyle);
		assertEquals("Thrower", runs.get(2).text);
		assertEquals(TextStyle.HOME, runs.get(2).textStyle);
		assertEquals(" resists the hunger.", runs.get(3).text);
	}

	@Test
	public void rendersFailureUsesGenitive() {
		given(game.getActingPlayer().getPlayer()).willReturn(thrower);
		given(thrower.getName()).willReturn("Thrower");
		given(thrower.getPlayerGender()).willReturn(PlayerGender.FEMALE);
		given(game.getTeamHome().hasPlayer(thrower)).willReturn(true);

		ReportAlwaysHungryRoll report = new ReportAlwaysHungryRoll("p1", false, 2, 4, false, null);
		List<Run> runs = render(new AlwaysHungryMessage(), report);

		assertEquals(" tries to eat her team-mate.", runs.get(3).text);
	}

	@Test
	public void rendersAwayThrower() {
		given(game.getActingPlayer().getPlayer()).willReturn(thrower);
		given(thrower.getName()).willReturn("AwayThrower");
		given(thrower.getPlayerGender()).willReturn(PlayerGender.NEUTRAL);
		given(game.getTeamHome().hasPlayer(thrower)).willReturn(false);

		ReportAlwaysHungryRoll report = new ReportAlwaysHungryRoll("p2", false, 2, 4, false, null);
		List<Run> runs = render(new AlwaysHungryMessage(), report);

		assertEquals(TextStyle.AWAY, runs.get(2).textStyle);
		assertEquals(" tries to eat its team-mate.", runs.get(3).text);
	}
}
