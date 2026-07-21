package com.fumbbl.ffb.client.report;

import com.fumbbl.ffb.FactoryType;
import com.fumbbl.ffb.client.TextStyle;
import com.fumbbl.ffb.mechanics.Mechanic;
import com.fumbbl.ffb.mechanics.bb2025.AgilityMechanic;
import com.fumbbl.ffb.model.Player;
import com.fumbbl.ffb.report.ReportSafeThrowRoll;
import org.junit.jupiter.api.Test;
import org.mockito.Mock;

import java.util.List;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.BDDMockito.given;

class SafeThrowRollMessageTest extends ReportMessageTestBase {

	@SuppressWarnings("rawtypes")
	@Mock
	private Player player;

	private void givenActingPlayer() {
		given(game.getActingPlayer().getPlayer()).willReturn(player);
		given(player.getName()).willReturn("Thrower");
		given(player.getAgilityWithModifiers()).willReturn(3);
		given(game.getTeamHome().hasPlayer(player)).willReturn(true);
	}

	private void stubBb2025Mechanic() {
		given(game.getRules().getFactory(FactoryType.Factory.MECHANIC).forName(Mechanic.Type.AGILITY.name()))
			.willReturn(new AgilityMechanic());
	}

	@Test
	public void renderSuccessfulThrowWithNeededRoll() {
		givenActingPlayer();
		stubBb2025Mechanic();

		ReportSafeThrowRoll report = new ReportSafeThrowRoll(null, true, 4, 2, false, null);
		List<Run> runs = render(new SafeThrowRollMessage(), report);

		assertEquals("Safe Throw Roll [ 4 ]", runs.get(0).text);
		assertEquals(" throws safely over any interceptors.", runs.get(3).text);
		Run needed = runs.stream().filter(r -> r.textStyle == TextStyle.NEEDED_ROLL).findFirst().orElseThrow();
		assertEquals("Succeeded on a roll of 2+ (Roll >= 3+)", needed.text);
	}

	@Test
	public void renderFailedThrowWithNeededRoll() {
		givenActingPlayer();
		stubBb2025Mechanic();

		ReportSafeThrowRoll report = new ReportSafeThrowRoll(null, false, 1, 2, false, null);
		List<Run> runs = render(new SafeThrowRollMessage(), report);

		assertEquals("'s Safe Throw fails to stop the interception.", runs.get(3).text);
		Run needed = runs.stream().filter(r -> r.textStyle == TextStyle.NEEDED_ROLL).findFirst().orElseThrow();
		assertEquals("Roll a 2+ to succeed (Roll >= 3+)", needed.text);
	}

	@Test
	public void renderReRolledOmitsNeededRoll() {
		givenActingPlayer();

		ReportSafeThrowRoll report = new ReportSafeThrowRoll(null, true, 4, 2, true, null);
		List<Run> runs = render(new SafeThrowRollMessage(), report);

		assertTrue(runs.stream().noneMatch(r -> r.textStyle == TextStyle.NEEDED_ROLL));
	}

	@Test
	public void renderBb2016UsesBb2016NeededRollFormat() {
		givenActingPlayer();
		given(game.getRules().getFactory(FactoryType.Factory.MECHANIC).forName(Mechanic.Type.AGILITY.name()))
			.willReturn(new com.fumbbl.ffb.mechanics.bb2016.AgilityMechanic());

		ReportSafeThrowRoll report = new ReportSafeThrowRoll(null, true, 4, 2, false, null);
		List<Run> runs = render(new SafeThrowRollMessage(), report);

		Run needed = runs.stream().filter(r -> r.textStyle == TextStyle.NEEDED_ROLL).findFirst().orElseThrow();
		assertEquals("Succeeded on a roll of 2+ (AG 3 + Roll > 6).", needed.text);
	}
}
