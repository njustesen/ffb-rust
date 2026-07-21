package com.fumbbl.ffb.client.report.mixed;

import com.fumbbl.ffb.FactoryType;
import com.fumbbl.ffb.client.TextStyle;
import com.fumbbl.ffb.client.report.ReportMessageTestBase;
import com.fumbbl.ffb.mechanics.Mechanic;
import com.fumbbl.ffb.mechanics.bb2020.AgilityMechanic;
import com.fumbbl.ffb.model.Player;
import com.fumbbl.ffb.modifiers.ModifierType;
import com.fumbbl.ffb.modifiers.PickupModifier;
import com.fumbbl.ffb.modifiers.RollModifier;
import com.fumbbl.ffb.report.mixed.ReportPickupRoll;
import org.junit.jupiter.api.Test;
import org.mockito.Mock;

import java.util.List;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.mockito.BDDMockito.given;

class PickUpRollMessageTest extends ReportMessageTestBase {

	@SuppressWarnings("rawtypes")
	@Mock
	private Player player;

	private void stubMechanic() {
		given(game.getRules().getFactory(FactoryType.Factory.MECHANIC).forName(Mechanic.Type.AGILITY.name()))
			.willReturn(new AgilityMechanic());
	}

	@Test
	public void firstAttemptSuccessReportsIntroAndNeededRoll() {
		given(game.getPlayerById("p1")).willReturn(player);
		given(player.getName()).willReturn("Grobnik");
		given(game.getTeamHome().hasPlayer(player)).willReturn(true);
		stubMechanic();

		ReportPickupRoll report = new ReportPickupRoll("p1", true, 5, 3, false, new RollModifier<?>[0]);
		List<Run> runs = render(new PickUpRollMessage(), report);

		assertEquals("Grobnik", runs.get(0).text);
		assertEquals(" tries to pick up the ball:", runs.get(1).text);
		assertEquals("Pickup Roll [ 5 ]", runs.get(3).text);
		assertEquals("Grobnik", runs.get(5).text);
		assertEquals(" picks up the ball.", runs.get(6).text);
		assertEquals(TextStyle.NEEDED_ROLL, runs.get(8).textStyle);
		assertEquals(true, runs.get(8).text.startsWith("Succeeded on a roll of 3+"));
	}

	@Test
	public void firstAttemptFailureReportsDropAndNeededRoll() {
		given(game.getPlayerById("p1")).willReturn(player);
		given(player.getName()).willReturn("Grobnik");
		given(game.getTeamHome().hasPlayer(player)).willReturn(true);
		stubMechanic();

		ReportPickupRoll report = new ReportPickupRoll("p1", false, 1, 4, false, new RollModifier<?>[0]);
		List<Run> runs = render(new PickUpRollMessage(), report);

		assertEquals(" drops the ball.", runs.get(6).text);
		assertEquals(true, runs.get(8).text.startsWith("Roll a 4+ to succeed"));
	}

	@Test
	public void reRolledSkipsIntroAndNeededRoll() {
		given(game.getPlayerById("p1")).willReturn(player);
		given(player.getName()).willReturn("Grobnik");
		given(game.getTeamHome().hasPlayer(player)).willReturn(true);

		ReportPickupRoll report = new ReportPickupRoll("p1", true, 5, 3, true, new RollModifier<?>[0]);
		List<Run> runs = render(new PickUpRollMessage(), report);

		// No intro line, no needed-roll line: only the "Pickup Roll" header + player + result.
		assertEquals(5, runs.size());
		assertEquals("Pickup Roll [ 5 ]", runs.get(0).text);
	}

	@Test
	public void rollModifiersAppendedToNeededRoll() {
		given(game.getPlayerById("p1")).willReturn(player);
		given(player.getName()).willReturn("Grobnik");
		given(game.getTeamHome().hasPlayer(player)).willReturn(true);
		given(player.getAgilityWithModifiers()).willReturn(3);
		stubMechanic();

		RollModifier<?>[] modifiers = new RollModifier<?>[] { new PickupModifier("TackleZone", 1, ModifierType.TACKLEZONE) };
		ReportPickupRoll report = new ReportPickupRoll("p1", true, 5, 3, false, modifiers);
		List<Run> runs = render(new PickUpRollMessage(), report);

		// java: AgilityMechanic.formatPickupResult formats the full RollModifier set with
		// numeric signs, unlike the Rust name-only approximation — real Java output verified here.
		assertEquals("Succeeded on a roll of 3+ (Roll - TackleZone >= 3+)", runs.get(8).text);
	}
}
