package com.fumbbl.ffb.client.report.bb2020;

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
import com.fumbbl.ffb.report.logcontrol.SkipInjuryParts;
import com.fumbbl.ffb.report.mixed.ReportInjury;
import org.junit.jupiter.api.Test;
import org.mockito.Mock;

import java.util.Collections;
import java.util.HashSet;
import java.util.List;
import java.util.Set;

import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.BDDMockito.given;

class InjuryMessageTest extends ReportMessageTestBase {

	@SuppressWarnings("rawtypes")
	@Mock
	private Player defender;

	@SuppressWarnings("rawtypes")
	@Mock
	private Player attacker;

	@SuppressWarnings("rawtypes")
	@Mock
	private ZappedPlayer zappedDefender;

	@Mock
	private SeriousInjury seriousInjury;

	private ReportInjury makeReport(boolean armorBroken, int[] armorRoll, int[] injuryRoll, int[] casualtyRoll,
									 Set<CasualtyModifier> casualtyModifiers, SeriousInjury injurySeriousInjury,
									 PlayerState injury) {
		return new ReportInjury("def1", new Block(), armorBroken, new ArmorModifier[0], armorRoll,
			new InjuryModifier[0], injuryRoll, casualtyRoll, injurySeriousInjury, new int[0], null, injury, null,
			"att1", casualtyModifiers, null, SkipInjuryParts.NONE);
	}

	@Test
	public void armourSavedWhenNotBroken() {
		given(game.getPlayerById("def1")).willReturn(defender);
		given(game.getPlayerById("att1")).willReturn(attacker);
		given(defender.getPlayerGender()).willReturn(PlayerGender.MALE);
		given(game.getTeamHome().hasPlayer(defender)).willReturn(true);

		ReportInjury report = makeReport(false, new int[]{3, 4}, new int[0], new int[0], Collections.emptySet(), null, null);
		List<Run> runs = render(new InjuryMessage(), report);
		List<String> texts = runs.stream().map(r -> r.text).collect(java.util.stream.Collectors.toList());

		assertTrue(texts.contains("Armour Roll [ 3 ][ 4 ]"));
		assertTrue(texts.stream().anyMatch(t -> t != null && t.contains("has been saved by his armour.")));
	}

	@Test
	public void armourBrokenInjuryRollNoCasualty() {
		given(game.getPlayerById("def1")).willReturn(defender);
		given(game.getPlayerById("att1")).willReturn(attacker);
		given(defender.getPlayerGender()).willReturn(PlayerGender.MALE);
		given(game.getTeamHome().hasPlayer(defender)).willReturn(true);

		ReportInjury report = makeReport(true, new int[]{5, 6}, new int[]{2, 3}, new int[0], Collections.emptySet(), null, null);
		List<Run> runs = render(new InjuryMessage(), report);
		List<String> texts = runs.stream().map(r -> r.text).collect(java.util.stream.Collectors.toList());

		assertTrue(texts.stream().anyMatch(t -> t != null && t.contains("has been broken.")));
		assertTrue(texts.contains("Injury Roll [ 2 ][ 3 ]"));
		assertFalse(texts.stream().anyMatch(t -> t != null && t.contains("suffers a casualty")));
	}

	@Test
	public void armourBrokenCasualtyWithModifiersAndSeriousInjury() {
		given(game.getPlayerById("def1")).willReturn(defender);
		given(game.getPlayerById("att1")).willReturn(attacker);
		given(defender.getPlayerGender()).willReturn(PlayerGender.MALE);
		given(game.getTeamHome().hasPlayer(defender)).willReturn(true);
		given(seriousInjury.showSiRoll()).willReturn(true);
		given(seriousInjury.getDescription()).willReturn("suffered a head injury (-1 AV)");

		Set<CasualtyModifier> modifiers = new HashSet<>();
		modifiers.add(new CasualtyModifier("Mighty Blow +1", 1));

		ReportInjury report = makeReport(true, new int[]{5, 6}, new int[]{4, 5}, new int[]{7, 3}, modifiers,
			seriousInjury, new PlayerState(PlayerState.BADLY_HURT));
		List<Run> runs = render(new InjuryMessage(), report);
		List<String> texts = runs.stream().map(r -> r.text).collect(java.util.stream.Collectors.toList());

		assertTrue(texts.stream().anyMatch(t -> t != null && t.contains("suffers a casualty.")));
		assertTrue(texts.contains("Casualty Roll [ 7 ]"));
		assertTrue(texts.stream().anyMatch(t -> t != null && t.contains("Rolled 7 + 1 Mighty Blow +1 = 8")));
		assertTrue(texts.stream().anyMatch(t -> t != null && t.contains("has been badly hurt.")));
		assertTrue(texts.contains("Lasting Injury Roll [ 3 ]"));
		assertTrue(texts.stream().anyMatch(t -> t != null && t.contains("suffered a head injury (-1 AV).")));
	}

	@Test
	public void zappedDefenderReportsAutomaticCasualty() {
		given(game.getPlayerById("def1")).willReturn((com.fumbbl.ffb.model.Player) zappedDefender);
		given(game.getPlayerById("att1")).willReturn(attacker);
		given(zappedDefender.getName()).willReturn("Defender");
		given(zappedDefender.getPlayerGender()).willReturn(PlayerGender.MALE);
		given(game.getTeamHome().hasPlayer(zappedDefender)).willReturn(true);

		ReportInjury report = makeReport(true, new int[]{5, 6}, new int[]{4, 5}, new int[]{7, 3}, Collections.emptySet(), null, null);
		List<Run> runs = render(new InjuryMessage(), report);
		List<String> texts = runs.stream().map(r -> r.text).collect(java.util.stream.Collectors.toList());

		assertTrue(texts.stream().anyMatch(t -> t != null && t.contains("is badly hurt automatically because he has been zapped.")));
	}
}
