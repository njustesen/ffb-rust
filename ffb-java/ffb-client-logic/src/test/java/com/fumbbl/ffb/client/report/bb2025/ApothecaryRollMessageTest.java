package com.fumbbl.ffb.client.report.bb2025;

import com.fumbbl.ffb.PlayerState;
import com.fumbbl.ffb.SeriousInjury;
import com.fumbbl.ffb.client.report.ReportMessageTestBase;
import com.fumbbl.ffb.model.Player;
import com.fumbbl.ffb.modifiers.bb2020.CasualtyModifier;
import com.fumbbl.ffb.report.ReportId;
import com.fumbbl.ffb.report.mixed.ReportApothecaryRoll;
import org.junit.jupiter.api.Test;
import org.mockito.Mock;

import java.util.Collections;
import java.util.HashSet;
import java.util.List;
import java.util.Set;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.BDDMockito.given;

class ApothecaryRollMessageTest extends ReportMessageTestBase {

	@SuppressWarnings("rawtypes")
	@Mock
	private Player player;

	@Mock
	private SeriousInjury seriousInjury;

	@Test
	public void reportIdIsApothecaryRoll() {
		assertEquals(ReportId.APOTHECARY_ROLL.getKey(), new ApothecaryRollMessage().getKey());
	}

	@Test
	public void emptyCasualtyRollRendersNothing() {
		ReportApothecaryRoll report = new ReportApothecaryRoll("p1", new int[]{}, null, null, null, Collections.emptySet());
		List<Run> runs = render(new ApothecaryRollMessage(), report);
		assertTrue(runs.isEmpty());
	}

	@Test
	public void basicCasualtyRollReportsUsedAndInjuryDescription() {
		given(game.getPlayerById("p1")).willReturn(player);
		given(game.getTeamHome().hasPlayer(player)).willReturn(true);

		ReportApothecaryRoll report = new ReportApothecaryRoll("p1", new int[]{4},
			new PlayerState(PlayerState.BADLY_HURT), null, null, Collections.emptySet());
		List<Run> runs = render(new ApothecaryRollMessage(), report);
		List<String> texts = runs.stream().map(r -> r.text).collect(java.util.stream.Collectors.toList());
		assertTrue(texts.stream().anyMatch(t -> "Apothecary used.".equals(t)));
		assertTrue(texts.stream().anyMatch(t -> "Casualty Roll [ 4 ]".equals(t)));
		assertTrue(texts.stream().anyMatch(t -> " has been badly hurt.".equals(t)));
	}

	@Test
	public void casualtyModifiersReportedSortedAlphabetically() {
		given(game.getPlayerById("p1")).willReturn(player);
		given(game.getTeamHome().hasPlayer(player)).willReturn(true);

		Set<CasualtyModifier> modifiers = new HashSet<>();
		modifiers.add(new CasualtyModifier("Zealous", 1));
		modifiers.add(new CasualtyModifier("Claws", 1));

		ReportApothecaryRoll report = new ReportApothecaryRoll("p1", new int[]{4},
			new PlayerState(PlayerState.BADLY_HURT), null, null, modifiers);
		List<Run> runs = render(new ApothecaryRollMessage(), report);
		List<String> texts = runs.stream().map(r -> r.text).collect(java.util.stream.Collectors.toList());
		String rolledLine = texts.stream().filter(t -> t != null && t.startsWith("Rolled")).findFirst().orElseThrow();
		assertTrue(rolledLine.indexOf("Claws") < rolledLine.indexOf("Zealous"));
	}

	@Test
	public void casualtyModifiersPresentPrintsSumLine() {
		given(game.getPlayerById("p1")).willReturn(player);
		given(game.getTeamHome().hasPlayer(player)).willReturn(true);

		Set<CasualtyModifier> modifiers = new HashSet<>();
		modifiers.add(new CasualtyModifier("Mighty Blow", 1));
		modifiers.add(new CasualtyModifier("Thick Skull", -1));

		ReportApothecaryRoll report = new ReportApothecaryRoll("p1", new int[]{3},
			new PlayerState(PlayerState.KNOCKED_OUT), null, null, modifiers);
		List<Run> runs = render(new ApothecaryRollMessage(), report);
		List<String> texts = runs.stream().map(r -> r.text).collect(java.util.stream.Collectors.toList());
		String rolledLine = texts.stream().filter(t -> t != null && t.startsWith("Rolled")).findFirst().orElseThrow();
		assertTrue(rolledLine.contains("1 Mighty Blow"));
		assertTrue(rolledLine.contains("-1 Thick Skull"));
		assertTrue(rolledLine.endsWith(" = 3"));
	}

	@Test
	public void seriousInjuryPresentReportsDescription() {
		given(game.getPlayerById("p1")).willReturn(player);
		given(game.getTeamHome().hasPlayer(player)).willReturn(true);
		given(player.getName()).willReturn("Grubb");
		given(seriousInjury.showSiRoll()).willReturn(true);
		given(seriousInjury.getDescription()).willReturn("suffers a broken leg");

		ReportApothecaryRoll report = new ReportApothecaryRoll("p1", new int[]{4, 8},
			new PlayerState(PlayerState.SERIOUS_INJURY), seriousInjury, null, Collections.emptySet());
		List<Run> runs = render(new ApothecaryRollMessage(), report);
		List<String> texts = runs.stream().map(r -> r.text).collect(java.util.stream.Collectors.toList());
		assertTrue(texts.stream().anyMatch(t -> " suffers a broken leg.".equals(t)));
	}
}
