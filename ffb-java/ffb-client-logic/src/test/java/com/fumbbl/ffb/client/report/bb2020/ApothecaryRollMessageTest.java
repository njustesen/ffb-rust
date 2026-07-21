package com.fumbbl.ffb.client.report.bb2020;

import com.fumbbl.ffb.PlayerState;
import com.fumbbl.ffb.SeriousInjury;
import com.fumbbl.ffb.client.report.ReportMessageTestBase;
import com.fumbbl.ffb.model.Player;
import com.fumbbl.ffb.modifiers.bb2020.CasualtyModifier;
import com.fumbbl.ffb.report.mixed.ReportApothecaryRoll;
import org.junit.jupiter.api.Test;
import org.mockito.Mock;

import java.util.Collections;
import java.util.HashSet;
import java.util.List;
import java.util.Set;

import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.BDDMockito.given;

class ApothecaryRollMessageTest extends ReportMessageTestBase {

	@SuppressWarnings("rawtypes")
	@Mock
	private Player player;

	@Mock
	private SeriousInjury seriousInjury;

	@Mock
	private SeriousInjury originalInjury;

	@Test
	public void noCasualtyRollRendersNothing() {
		ReportApothecaryRoll report = new ReportApothecaryRoll("p1", new int[]{}, null, null, null, Collections.emptySet());
		List<Run> runs = render(new ApothecaryRollMessage(), report);
		assertTrue(runs.isEmpty());
	}

	@Test
	public void casualtyRollWithoutSeriousInjuryPrintsStateDescription() {
		given(game.getPlayerById("p1")).willReturn(player);
		given(game.getTeamHome().hasPlayer(player)).willReturn(true);

		ReportApothecaryRoll report = new ReportApothecaryRoll("p1", new int[]{3},
			new PlayerState(PlayerState.KNOCKED_OUT), null, null, Collections.emptySet());
		List<Run> runs = render(new ApothecaryRollMessage(), report);
		List<String> texts = runs.stream().map(r -> r.text).collect(java.util.stream.Collectors.toList());

		assertTrue(texts.stream().anyMatch(t -> t != null && t.contains("Apothecary used.")));
		assertTrue(texts.stream().anyMatch(t -> t != null && t.contains("Casualty Roll [ 3 ]")));
		assertTrue(texts.stream().anyMatch(t -> t != null && t.contains("has been knocked out.")));
	}

	@Test
	public void seriousInjuryPresentPrintsDescriptionAndShowsSiRoll() {
		given(game.getPlayerById("p1")).willReturn(player);
		given(game.getTeamHome().hasPlayer(player)).willReturn(true);
		given(seriousInjury.showSiRoll()).willReturn(true);
		given(seriousInjury.getDescription()).willReturn("suffered a head injury (-1 AV)");

		ReportApothecaryRoll report = new ReportApothecaryRoll("p1", new int[]{3, 5},
			new PlayerState(PlayerState.KNOCKED_OUT), seriousInjury, null, Collections.emptySet());
		List<Run> runs = render(new ApothecaryRollMessage(), report);
		List<String> texts = runs.stream().map(r -> r.text).collect(java.util.stream.Collectors.toList());

		assertTrue(texts.stream().anyMatch(t -> t != null && t.contains("Casualty Roll [ 3 ][ 5 ]")));
		assertTrue(texts.stream().anyMatch(t -> t != null && t.contains("suffered a head injury")));
	}

	@Test
	public void originalInjuryPresentPrintsExplanation() {
		given(game.getPlayerById("p1")).willReturn(player);
		given(game.getTeamHome().hasPlayer(player)).willReturn(true);
		given(seriousInjury.showSiRoll()).willReturn(true);
		given(seriousInjury.getDescription()).willReturn("suffered a head injury (-1 AV)");
		given(originalInjury.getDescription()).willReturn("suffered a neck injury (-1 AG)");

		ReportApothecaryRoll report = new ReportApothecaryRoll("p1", new int[]{3, 5},
			new PlayerState(PlayerState.KNOCKED_OUT), seriousInjury, originalInjury, Collections.emptySet());
		List<Run> runs = render(new ApothecaryRollMessage(), report);
		List<String> texts = runs.stream().map(r -> r.text).collect(java.util.stream.Collectors.toList());

		assertTrue(texts.stream().anyMatch(t -> t != null && t.contains("would have suffered a neck injury")));
		assertTrue(texts.stream().anyMatch(t -> t != null && t.contains("but that stat cannot be reduced any further")));
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

		assertTrue(texts.stream().anyMatch(t -> t != null && t.contains("Rolled 3")));
		assertTrue(texts.stream().anyMatch(t -> t != null && t.contains("-1 Thick Skull")));
		assertTrue(texts.stream().anyMatch(t -> t != null && t.contains("1 Mighty Blow")));
		assertTrue(texts.stream().anyMatch(t -> t != null && t.contains(" = 3")));
	}
}
