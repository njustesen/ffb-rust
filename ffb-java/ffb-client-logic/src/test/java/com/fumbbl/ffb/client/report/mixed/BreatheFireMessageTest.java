package com.fumbbl.ffb.client.report.mixed;

import com.fumbbl.ffb.BreatheFireResult;
import com.fumbbl.ffb.PlayerGender;
import com.fumbbl.ffb.client.report.ReportMessageTestBase;
import com.fumbbl.ffb.model.Player;
import com.fumbbl.ffb.report.mixed.ReportBreatheFire;
import org.junit.jupiter.api.Test;
import org.mockito.Mock;

import java.util.List;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.mockito.BDDMockito.given;

class BreatheFireMessageTest extends ReportMessageTestBase {

	@SuppressWarnings("rawtypes")
	@Mock
	private Player thrower;

	@SuppressWarnings("rawtypes")
	@Mock
	private Player defender;

	@Test
	public void knockDownEngulfsDefender() {
		given(game.getActingPlayer().getPlayer()).willReturn(thrower);
		given(game.getTeamHome().hasPlayer(thrower)).willReturn(true);
		given(game.getPlayerById("p2")).willReturn(defender);
		given(defender.getName()).willReturn("Victim");

		ReportBreatheFire report = new ReportBreatheFire(null, true, 6, 2, false, "p2", BreatheFireResult.KNOCK_DOWN, false);
		List<Run> runs = render(new BreatheFireMessage(), report);

		assertEquals("Breathe Fire Roll [ 6 ]", runs.get(0).text);
		assertEquals(" engulfs ", runs.get(3).text);
		assertEquals("Victim", runs.get(4).text);
		assertEquals(" in flames.", runs.get(5).text);
	}

	@Test
	public void proneForcesCoverAndPrintsNeededRoll() {
		given(game.getActingPlayer().getPlayer()).willReturn(thrower);
		given(game.getTeamHome().hasPlayer(thrower)).willReturn(true);
		given(game.getPlayerById("p2")).willReturn(defender);

		ReportBreatheFire report = new ReportBreatheFire(null, true, 4, 2, false, "p2", BreatheFireResult.PRONE, true);
		List<Run> runs = render(new BreatheFireMessage(), report);

		assertEquals(" to take cover.", runs.get(5).text);
		assertEquals(" (Roll - 1 opponent has strength 5 or more >= 6 to knock down opponent).", runs.get(7).text);
	}

	@Test
	public void noEffectPrintsTwoNeededRolls() {
		given(game.getActingPlayer().getPlayer()).willReturn(thrower);
		given(game.getTeamHome().hasPlayer(thrower)).willReturn(true);
		given(game.getPlayerById("p2")).willReturn(defender);

		ReportBreatheFire report = new ReportBreatheFire(null, false, 3, 2, false, "p2", BreatheFireResult.NO_EFFECT, false);
		List<Run> runs = render(new BreatheFireMessage(), report);

		assertEquals(".", runs.get(5).text);
		assertEquals(" (Roll >= 6 to knock down opponent).", runs.get(7).text);
		assertEquals(" (Roll >= 4 to place opponent prone).", runs.get(9).text);
	}

	@Test
	public void failureEngulfsSelfAndPrintsThreeNeededRolls() {
		given(game.getActingPlayer().getPlayer()).willReturn(thrower);
		given(game.getTeamHome().hasPlayer(thrower)).willReturn(true);
		given(thrower.getPlayerGender()).willReturn(PlayerGender.MALE);
		given(game.getPlayerById("p2")).willReturn(defender);

		ReportBreatheFire report = new ReportBreatheFire(null, false, 1, 2, false, "p2", BreatheFireResult.FAILURE, false);
		List<Run> runs = render(new BreatheFireMessage(), report);

		assertEquals(" engulfs himself in flames.", runs.get(3).text);
		assertEquals(" (Roll >= 6 to knock down opponent).", runs.get(5).text);
		assertEquals(" (Roll >= 4 to place opponent prone).", runs.get(7).text);
		assertEquals(" (Roll >= 2 to avoid knock down).", runs.get(9).text);
	}
}
