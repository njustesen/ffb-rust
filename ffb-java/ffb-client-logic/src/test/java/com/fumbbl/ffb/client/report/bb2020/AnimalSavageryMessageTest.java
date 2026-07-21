package com.fumbbl.ffb.client.report.bb2020;

import com.fumbbl.ffb.PlayerGender;
import com.fumbbl.ffb.client.report.ReportMessageTestBase;
import com.fumbbl.ffb.model.Player;
import com.fumbbl.ffb.report.mixed.ReportAnimalSavagery;
import org.junit.jupiter.api.Test;
import org.mockito.Mock;

import java.util.List;
import java.util.stream.Collectors;

import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.BDDMockito.given;

class AnimalSavageryMessageTest extends ReportMessageTestBase {

	@SuppressWarnings("rawtypes")
	@Mock
	private Player attacker;

	@SuppressWarnings("rawtypes")
	@Mock
	private Player defender;

	@Test
	public void lashesOutAgainstDefender() {
		given(game.getPlayerById("attacker")).willReturn(attacker);
		given(attacker.getName()).willReturn("Player attacker");
		given(game.getTeamHome().hasPlayer(attacker)).willReturn(true);
		given(game.getPlayerById("defender")).willReturn(defender);
		given(defender.getName()).willReturn("Player defender");
		given(game.getTeamHome().hasPlayer(defender)).willReturn(false);

		ReportAnimalSavagery report = new ReportAnimalSavagery("attacker", "defender");
		List<Run> runs = render(new AnimalSavageryMessage(), report);
		List<String> texts = runs.stream().map(r -> r.text).collect(Collectors.toList());

		assertTrue(texts.contains("Player attacker"));
		assertTrue(texts.stream().anyMatch(t -> t.contains(" lashes out against ")));
		assertTrue(texts.contains("Player defender"));
		assertTrue(texts.contains("."));
	}

	@Test
	public void noDefenderLosesGenitiveAction() {
		given(game.getPlayerById("attacker")).willReturn(attacker);
		given(attacker.getName()).willReturn("Player attacker");
		given(game.getTeamHome().hasPlayer(attacker)).willReturn(true);
		given(attacker.getPlayerGender()).willReturn(PlayerGender.MALE);

		ReportAnimalSavagery report = new ReportAnimalSavagery("attacker");
		List<Run> runs = render(new AnimalSavageryMessage(), report);

		assertTrue(runs.stream().anyMatch(r -> r.text.contains("loses his action")));
	}

	@Test
	public void noDefenderUsesFemaleGenitive() {
		given(game.getPlayerById("attacker")).willReturn(attacker);
		given(attacker.getName()).willReturn("Player attacker");
		given(game.getTeamHome().hasPlayer(attacker)).willReturn(true);
		given(attacker.getPlayerGender()).willReturn(PlayerGender.FEMALE);

		ReportAnimalSavagery report = new ReportAnimalSavagery("attacker");
		List<Run> runs = render(new AnimalSavageryMessage(), report);

		assertTrue(runs.stream().anyMatch(r -> r.text.contains("loses her action")));
	}

	@Test
	public void emptyDefenderIdIsTreatedAsAbsent() {
		given(game.getPlayerById("attacker")).willReturn(attacker);
		given(attacker.getName()).willReturn("Player attacker");
		given(game.getTeamHome().hasPlayer(attacker)).willReturn(true);
		given(attacker.getPlayerGender()).willReturn(PlayerGender.MALE);

		ReportAnimalSavagery report = new ReportAnimalSavagery("attacker", "");
		List<Run> runs = render(new AnimalSavageryMessage(), report);

		assertTrue(runs.stream().anyMatch(r -> r.text.contains("has no one to lash out against")));
	}
}
