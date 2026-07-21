package com.fumbbl.ffb.client.report;

import com.fumbbl.ffb.FactoryType;
import com.fumbbl.ffb.PlayerGender;
import com.fumbbl.ffb.client.TextStyle;
import com.fumbbl.ffb.mechanics.Mechanic;
import com.fumbbl.ffb.mechanics.bb2025.AgilityMechanic;
import com.fumbbl.ffb.model.Player;
import com.fumbbl.ffb.report.ReportRightStuffRoll;
import org.junit.jupiter.api.Test;
import org.mockito.Mock;

import java.util.List;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.BDDMockito.given;

class RightStuffRollMessageTest extends ReportMessageTestBase {

	@SuppressWarnings("rawtypes")
	@Mock
	private Player player;

	private void givenThrownPlayer() {
		given(game.getPlayerById("p1")).willReturn(player);
		given(player.getName()).willReturn("Gutter Runner");
		given(player.getPlayerGender()).willReturn(PlayerGender.MALE);
		given(player.getAgilityWithModifiers()).willReturn(3);
		given(game.getTeamHome().hasPlayer(player)).willReturn(true);
	}

	private void stubBb2025Mechanic() {
		given(game.getRules().getFactory(FactoryType.Factory.MECHANIC).forName(Mechanic.Type.AGILITY.name()))
			.willReturn(new AgilityMechanic());
	}

	@Test
	public void renderSuccessfulLandsOnFeetWithNeededRoll() {
		givenThrownPlayer();
		stubBb2025Mechanic();

		ReportRightStuffRoll report = new ReportRightStuffRoll("p1", true, 4, 2, false, null);
		List<Run> runs = render(new RightStuffRollMessage(), report);

		assertEquals("Landing Roll [ 4 ]", runs.get(0).text);
		assertEquals(" lands on his feet.", runs.get(3).text);
		Run needed = runs.stream().filter(r -> r.textStyle == TextStyle.NEEDED_ROLL).findFirst().orElseThrow();
		assertEquals("Succeeded on a roll of 2+ (Roll >= 3+)", needed.text);
	}

	@Test
	public void renderFailedCrashesWithNeededRoll() {
		givenThrownPlayer();
		stubBb2025Mechanic();

		ReportRightStuffRoll report = new ReportRightStuffRoll("p1", false, 1, 2, false, null);
		List<Run> runs = render(new RightStuffRollMessage(), report);

		assertEquals(" crashes to the ground.", runs.get(3).text);
		Run needed = runs.stream().filter(r -> r.textStyle == TextStyle.NEEDED_ROLL).findFirst().orElseThrow();
		assertEquals("Roll a 2+ to succeed (Roll >= 3+)", needed.text);
	}

	@Test
	public void renderReRolledOmitsNeededRoll() {
		givenThrownPlayer();

		ReportRightStuffRoll report = new ReportRightStuffRoll("p1", true, 4, 2, true, null);
		List<Run> runs = render(new RightStuffRollMessage(), report);

		assertTrue(runs.stream().noneMatch(r -> r.textStyle == TextStyle.NEEDED_ROLL));
	}

	@Test
	public void renderBb2016UsesBb2016NeededRollFormat() {
		givenThrownPlayer();
		given(game.getRules().getFactory(FactoryType.Factory.MECHANIC).forName(Mechanic.Type.AGILITY.name()))
			.willReturn(new com.fumbbl.ffb.mechanics.bb2016.AgilityMechanic());

		ReportRightStuffRoll report = new ReportRightStuffRoll("p1", true, 4, 2, false, null);
		List<Run> runs = render(new RightStuffRollMessage(), report);

		Run needed = runs.stream().filter(r -> r.textStyle == TextStyle.NEEDED_ROLL).findFirst().orElseThrow();
		assertEquals("Succeeded on a roll of 2+ (AG 3 + Roll > 6).", needed.text);
	}
}
