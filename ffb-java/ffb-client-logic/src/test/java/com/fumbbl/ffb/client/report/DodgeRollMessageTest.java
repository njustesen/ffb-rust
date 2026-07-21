package com.fumbbl.ffb.client.report;

import com.fumbbl.ffb.FactoryType;
import com.fumbbl.ffb.client.TextStyle;
import com.fumbbl.ffb.mechanics.Mechanic;
import com.fumbbl.ffb.mechanics.bb2025.AgilityMechanic;
import com.fumbbl.ffb.model.Player;
import com.fumbbl.ffb.modifiers.DodgeModifier;
import com.fumbbl.ffb.modifiers.ModifierType;
import com.fumbbl.ffb.modifiers.RollModifier;
import com.fumbbl.ffb.report.mixed.ReportDodgeRoll;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.mockito.Mock;

import java.util.Collections;
import java.util.List;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.BDDMockito.given;

class DodgeRollMessageTest extends ReportMessageTestBase {

	@SuppressWarnings("rawtypes")
	@Mock
	private Player player;

	@BeforeEach
	public void setUp() {
		given(game.getPlayerById("p1")).willReturn(player);
		given(player.getName()).willReturn("Dodger");
		given(game.getTeamHome().hasPlayer(player)).willReturn(true);
		given(player.getSkillsIncludingTemporaryOnes()).willReturn(Collections.emptySet());
	}

	private void stubMechanic() {
		given(game.getRules().getFactory(FactoryType.Factory.MECHANIC).forName(Mechanic.Type.AGILITY.name()))
			.willReturn(new AgilityMechanic());
	}

	@Test
	public void successfulDodgePrintsSuccessMessage() {
		stubMechanic();

		ReportDodgeRoll report = new ReportDodgeRoll("p1", true, 4, 3, false, null, null);
		List<Run> runs = render(new DodgeRollMessage(), report);

		assertTrue(runs.stream().anyMatch(r -> " dodges successfully.".equals(r.text)));
		assertEquals("Dodge Roll [ 4 ]", runs.get(0).text);
	}

	@Test
	public void failedDodgePrintsTripsMessageAndNeededRoll() {
		stubMechanic();

		ReportDodgeRoll report = new ReportDodgeRoll("p1", false, 1, 3, false, null, null);
		List<Run> runs = render(new DodgeRollMessage(), report);

		assertTrue(runs.stream().anyMatch(r -> " trips while dodging.".equals(r.text)));
		Run needed = runs.stream().filter(r -> r.textStyle == TextStyle.NEEDED_ROLL).findFirst().orElseThrow();
		assertTrue(needed.text.startsWith("Roll a 3+ to succeed"));
	}

	@Test
	public void zeroRollShowsNewDodgeResult() {
		stubMechanic();

		ReportDodgeRoll report = new ReportDodgeRoll("p1", true, 0, 3, false, null, null);
		List<Run> runs = render(new DodgeRollMessage(), report);

		assertEquals("New Dodge Result", runs.get(0).text);
	}

	@Test
	public void breakTackleModifierPrintsBreakFreeMessage() {
		stubMechanic();

		RollModifier<?>[] modifiers = new RollModifier<?>[] {
			new DodgeModifier("Break Tackle", 0, ModifierType.REGULAR, true)
		};
		ReportDodgeRoll report = new ReportDodgeRoll("p1", true, 4, 3, false, modifiers, null);
		List<Run> runs = render(new DodgeRollMessage(), report);

		assertTrue(runs.stream().anyMatch(r -> " uses Break Tackle to break free.".equals(r.text)));
	}

	@Test
	public void reRolledSkipsIntroLinesAndNeededRoll() {
		RollModifier<?>[] modifiers = new RollModifier<?>[] {
			new DodgeModifier("Break Tackle", 0, ModifierType.REGULAR, true)
		};
		ReportDodgeRoll report = new ReportDodgeRoll("p1", true, 4, 3, true, modifiers, null);
		List<Run> runs = render(new DodgeRollMessage(), report);

		assertTrue(runs.stream().noneMatch(r -> " uses Break Tackle to break free.".equals(r.text)));
		assertTrue(runs.stream().noneMatch(r -> r.textStyle == TextStyle.NEEDED_ROLL));
	}

	@Test
	public void reportIdIsDodgeRoll() {
		assertEquals("dodgeRoll", new DodgeRollMessage().getKey());
	}
}
