package com.fumbbl.ffb.client.report.bb2025;

import com.fumbbl.ffb.FactoryType;
import com.fumbbl.ffb.client.report.ReportMessageTestBase;
import com.fumbbl.ffb.mechanics.Mechanic;
import com.fumbbl.ffb.model.Player;
import com.fumbbl.ffb.modifiers.ModifierType;
import com.fumbbl.ffb.modifiers.PickupModifier;
import com.fumbbl.ffb.modifiers.RollModifier;
import com.fumbbl.ffb.report.ReportId;
import com.fumbbl.ffb.report.bb2025.ReportPickupRoll;
import org.junit.jupiter.api.Test;
import org.mockito.Mock;

import java.util.List;
import java.util.Objects;
import java.util.stream.Collectors;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.BDDMockito.given;

class PickUpRollMessageTest extends ReportMessageTestBase {

	@SuppressWarnings("rawtypes")
	@Mock
	private Player player;

	private void stubAgilityMechanic() {
		given(game.getRules().getFactory(FactoryType.Factory.MECHANIC).forName(Mechanic.Type.AGILITY.name()))
			.willReturn(new com.fumbbl.ffb.mechanics.bb2025.AgilityMechanic());
	}

	private void stubPlayer(int agility) {
		given(game.getPlayerById("p1")).willReturn(player);
		given(game.getTeamHome().hasPlayer(player)).willReturn(true);
		given(player.getName()).willReturn("Runner");
		given(player.getAgilityWithModifiers()).willReturn(agility);
	}

	@Test
	public void successfulPickupReportsSuccessAndNeededRoll() {
		stubAgilityMechanic();
		stubPlayer(3);
		ReportPickupRoll report = new ReportPickupRoll("p1", true, 4, 3, false, new RollModifier[0], false);
		List<Run> runs = render(new PickUpRollMessage(), report);
		List<String> texts = runs.stream().map(r -> r.text).filter(Objects::nonNull).collect(Collectors.toList());
		assertTrue(texts.stream().anyMatch(t -> t.contains("tries to pick up the ball:")));
		assertTrue(texts.stream().anyMatch(t -> t.contains("Pickup Roll [ 4 ]")));
		assertTrue(texts.stream().anyMatch(t -> t.contains("picks up the ball.")));
		assertTrue(texts.stream().anyMatch(t -> t.contains("Succeeded on a roll of 3+")));
	}

	@Test
	public void failedPickupReportsDropAndNeededRoll() {
		stubAgilityMechanic();
		stubPlayer(3);
		ReportPickupRoll report = new ReportPickupRoll("p1", false, 1, 3, false, new RollModifier[0], false);
		List<Run> runs = render(new PickUpRollMessage(), report);
		List<String> texts = runs.stream().map(r -> r.text).filter(Objects::nonNull).collect(Collectors.toList());
		assertTrue(texts.stream().anyMatch(t -> t.contains("drops the ball.")));
		assertTrue(texts.stream().anyMatch(t -> t.contains("Roll a 3+ to succeed")));
	}

	@Test
	public void reRolledPickupSkipsIntroAndNeededRoll() {
		stubAgilityMechanic();
		stubPlayer(3);
		ReportPickupRoll report = new ReportPickupRoll("p1", true, 4, 3, true, new RollModifier[0], false);
		List<Run> runs = render(new PickUpRollMessage(), report);
		List<String> texts = runs.stream().map(r -> r.text).filter(Objects::nonNull).collect(Collectors.toList());
		assertFalse(texts.stream().anyMatch(t -> t.contains("tries to pick up the ball:")));
		assertFalse(texts.stream().anyMatch(t -> t.contains("Succeeded on a roll of")));
		assertTrue(texts.stream().anyMatch(t -> t.contains("picks up the ball.")));
	}

	@Test
	public void secureTheBallUsesBaseRollOfTwo() {
		stubAgilityMechanic();
		stubPlayer(3);
		ReportPickupRoll report = new ReportPickupRoll("p1", true, 2, 2, false, new RollModifier[0], true);
		List<Run> runs = render(new PickUpRollMessage(), report);
		List<String> texts = runs.stream().map(r -> r.text).filter(Objects::nonNull).collect(Collectors.toList());
		assertTrue(texts.stream().anyMatch(t -> t.contains(">= 2+")));
	}

	@Test
	public void rollModifiersAreIncludedByName() {
		stubAgilityMechanic();
		stubPlayer(3);
		RollModifier<?>[] modifiers = new RollModifier[] { new PickupModifier("TackleZone", 0, ModifierType.TACKLEZONE) };
		ReportPickupRoll report = new ReportPickupRoll("p1", false, 1, 3, false, modifiers, false);
		List<Run> runs = render(new PickUpRollMessage(), report);
		List<String> texts = runs.stream().map(r -> r.text).filter(Objects::nonNull).collect(Collectors.toList());
		assertTrue(texts.stream().anyMatch(t -> t.contains("TackleZone")));
	}

	@Test
	public void reportIdIsPickUpRoll() {
		assertEquals(ReportId.PICK_UP_ROLL.getKey(), new PickUpRollMessage().getKey());
	}
}
