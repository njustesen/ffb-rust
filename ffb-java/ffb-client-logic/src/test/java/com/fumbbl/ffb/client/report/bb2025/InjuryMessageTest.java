package com.fumbbl.ffb.client.report.bb2025;

import com.fumbbl.ffb.PlayerGender;
import com.fumbbl.ffb.PlayerState;
import com.fumbbl.ffb.SeriousInjury;
import com.fumbbl.ffb.client.report.ReportMessageTestBase;
import com.fumbbl.ffb.injury.Block;
import com.fumbbl.ffb.model.Player;
import com.fumbbl.ffb.model.ZappedPlayer;
import com.fumbbl.ffb.modifiers.ArmorModifier;
import com.fumbbl.ffb.modifiers.InjuryModifier;
import com.fumbbl.ffb.modifiers.bb2020.CasualtyModifier;
import com.fumbbl.ffb.report.ReportId;
import com.fumbbl.ffb.report.logcontrol.SkipInjuryParts;
import com.fumbbl.ffb.report.mixed.ReportInjury;
import org.junit.jupiter.api.Test;
import org.mockito.Mock;

import java.util.Collections;
import java.util.List;
import java.util.Set;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.BDDMockito.given;

class InjuryMessageTest extends ReportMessageTestBase {

	@SuppressWarnings("rawtypes")
	@Mock
	private Player defender;

	@Mock
	private ZappedPlayer zappedDefender;

	private ReportInjury baseReport(boolean armorBroken, int[] armorRoll, int[] injuryRoll, int[] casualtyRoll,
	                                 PlayerState injury, SeriousInjury seriousInjury, SkipInjuryParts skip) {
		Set<CasualtyModifier> noCasualtyModifiers = Collections.emptySet();
		return new ReportInjury("d1", new Block(), armorBroken, new ArmorModifier[0], armorRoll,
			new InjuryModifier[0], injuryRoll, casualtyRoll, seriousInjury, new int[]{}, null, injury, null,
			"a1", noCasualtyModifiers, null, skip);
	}

	@Test
	public void armourSavedPrintsRolledTotalAndSavedLine() {
		given(game.getPlayerById("d1")).willReturn(defender);
		given(game.getTeamHome().hasPlayer(defender)).willReturn(false);
		given(defender.getPlayerGender()).willReturn(PlayerGender.FEMALE);

		ReportInjury report = baseReport(false, new int[]{4, 5}, new int[]{}, new int[]{}, null, null, SkipInjuryParts.NONE);
		List<Run> runs = render(new InjuryMessage(), report);
		List<String> texts = runs.stream().map(r -> r.text).collect(java.util.stream.Collectors.toList());
		assertTrue(texts.stream().anyMatch(t -> t != null && t.contains("Armour Roll [ 4 ][ 5 ]")));
		assertTrue(texts.stream().anyMatch(t -> t != null && t.contains("Rolled Total of 9")));
		assertTrue(texts.stream().anyMatch(t -> t != null && t.contains("has been saved by her armour.")));
	}

	@Test
	public void armourBrokenPrintsBrokenLineAndInjuryRoll() {
		given(game.getPlayerById("d1")).willReturn(defender);
		given(game.getTeamHome().hasPlayer(defender)).willReturn(false);
		given(defender.getPlayerGender()).willReturn(PlayerGender.FEMALE);

		ReportInjury report = baseReport(true, new int[]{4, 5}, new int[]{2, 3}, new int[]{},
			new PlayerState(PlayerState.KNOCKED_OUT), null, SkipInjuryParts.NONE);
		List<Run> runs = render(new InjuryMessage(), report);
		List<String> texts = runs.stream().map(r -> r.text).collect(java.util.stream.Collectors.toList());
		assertTrue(texts.stream().anyMatch(t -> t != null && t.contains("has been broken.")));
		assertTrue(texts.stream().anyMatch(t -> t != null && t.contains("Injury Roll [ 2 ][ 3 ]")));
		assertTrue(texts.stream().anyMatch(t -> t != null && t.contains("has been knocked out.")));
	}

	@Test
	public void casualtyRollReportsInjuryAndSeriousInjury() {
		given(game.getPlayerById("d1")).willReturn(defender);
		given(game.getTeamHome().hasPlayer(defender)).willReturn(false);
		given(defender.getPlayerGender()).willReturn(PlayerGender.FEMALE);

		SeriousInjury seriousInjury = org.mockito.Mockito.mock(SeriousInjury.class);
		given(seriousInjury.getDescription()).willReturn("suffers a broken leg");

		ReportInjury report = baseReport(true, new int[]{4, 5}, new int[]{5, 6}, new int[]{7, 42},
			new PlayerState(PlayerState.SERIOUS_INJURY), seriousInjury, SkipInjuryParts.NONE);
		List<Run> runs = render(new InjuryMessage(), report);
		List<String> texts = runs.stream().map(r -> r.text).collect(java.util.stream.Collectors.toList());
		assertTrue(texts.stream().anyMatch(t -> t != null && t.contains("suffers a casualty.")));
		assertTrue(texts.stream().anyMatch(t -> t != null && t.contains("Casualty Roll [ 7 ]")));
		assertTrue(texts.stream().anyMatch(t -> t != null && t.contains("has been seriously injured.")));
	}

	@Test
	public void zappedDefenderReportsAutomaticCas() {
		given(game.getPlayerById("d1")).willReturn((com.fumbbl.ffb.model.Player) zappedDefender);
		given(game.getTeamHome().hasPlayer(zappedDefender)).willReturn(false);
		given(zappedDefender.getName()).willReturn("Defender");
		given(zappedDefender.getPlayerGender()).willReturn(PlayerGender.FEMALE);

		ReportInjury report = baseReport(true, new int[]{4, 5}, new int[]{}, new int[]{7, 42}, null, null, SkipInjuryParts.NONE);
		List<Run> runs = render(new InjuryMessage(), report);
		List<String> texts = runs.stream().map(r -> r.text).collect(java.util.stream.Collectors.toList());
		assertTrue(texts.stream().anyMatch(t -> t != null && t.contains("is badly hurt automatically because she has been zapped.")));
	}

	@Test
	public void skipArmourSuppressesArmourRollLine() {
		given(game.getPlayerById("d1")).willReturn(defender);
		given(game.getTeamHome().hasPlayer(defender)).willReturn(false);
		given(defender.getPlayerGender()).willReturn(PlayerGender.FEMALE);

		ReportInjury report = baseReport(false, new int[]{4, 5}, new int[]{}, new int[]{}, null, null, SkipInjuryParts.ARMOUR);
		List<Run> runs = render(new InjuryMessage(), report);
		List<String> texts = runs.stream().map(r -> r.text).collect(java.util.stream.Collectors.toList());
		assertFalse(texts.stream().anyMatch(t -> t != null && t.contains("Armour Roll")));
	}

	@Test
	public void reportIdIsInjury() {
		assertEquals(ReportId.INJURY.getKey(), new InjuryMessage().getKey());
	}
}
