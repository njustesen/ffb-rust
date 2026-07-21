package com.fumbbl.ffb.client.report.bb2025;

import com.fumbbl.ffb.PlayerGender;
import com.fumbbl.ffb.client.report.ReportMessageTestBase;
import com.fumbbl.ffb.model.Player;
import com.fumbbl.ffb.option.GameOptionBoolean;
import com.fumbbl.ffb.option.GameOptionId;
import com.fumbbl.ffb.report.ReportId;
import com.fumbbl.ffb.report.mixed.ReportAnimalSavagery;
import org.junit.jupiter.api.Test;
import org.mockito.Mock;

import java.util.List;

import static org.junit.jupiter.api.Assertions.assertEquals;
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
		given(game.getPlayerById("a1")).willReturn(attacker);
		given(game.getPlayerById("d1")).willReturn(defender);
		given(game.getTeamHome().hasPlayer(attacker)).willReturn(true);
		given(game.getTeamHome().hasPlayer(defender)).willReturn(false);
		given(attacker.getPlayerGender()).willReturn(PlayerGender.MALE);
		given(game.getOptions().getOptionWithDefault(GameOptionId.ANIMAL_SAVAGERY_LASH_OUT_ENDS_ACTIVATION))
			.willReturn(new GameOptionBoolean(GameOptionId.ANIMAL_SAVAGERY_LASH_OUT_ENDS_ACTIVATION).setValue(false));

		ReportAnimalSavagery report = new ReportAnimalSavagery("a1", "d1");
		List<Run> runs = render(new AnimalSavageryMessage(), report);

		List<String> texts = runs.stream().map(r -> r.text).collect(java.util.stream.Collectors.toList());
		assertTrue(texts.contains(" lashes out against "));
		assertTrue(texts.stream().anyMatch(t -> ".".equals(t)));
	}

	@Test
	public void noDefenderLosesAction() {
		given(game.getPlayerById("a1")).willReturn(attacker);
		given(game.getTeamHome().hasPlayer(attacker)).willReturn(true);
		given(attacker.getPlayerGender()).willReturn(PlayerGender.MALE);

		ReportAnimalSavagery report = new ReportAnimalSavagery("a1", null);
		List<Run> runs = render(new AnimalSavageryMessage(), report);

		List<String> texts = runs.stream().map(r -> r.text).collect(java.util.stream.Collectors.toList());
		assertTrue(texts.stream().anyMatch(t -> t != null && t.contains("has no one to lash out against")));
		assertTrue(texts.stream().anyMatch(t -> t != null && t.contains("his action")));
	}

	@Test
	public void lashOutEndsActivationOptionAppendsText() {
		given(game.getPlayerById("a1")).willReturn(attacker);
		given(game.getPlayerById("d1")).willReturn(defender);
		given(game.getTeamHome().hasPlayer(attacker)).willReturn(true);
		given(game.getTeamHome().hasPlayer(defender)).willReturn(false);
		given(attacker.getPlayerGender()).willReturn(PlayerGender.MALE);
		given(game.getOptions().getOptionWithDefault(GameOptionId.ANIMAL_SAVAGERY_LASH_OUT_ENDS_ACTIVATION))
			.willReturn(new GameOptionBoolean(GameOptionId.ANIMAL_SAVAGERY_LASH_OUT_ENDS_ACTIVATION).setValue(true));

		ReportAnimalSavagery report = new ReportAnimalSavagery("a1", "d1");
		List<Run> runs = render(new AnimalSavageryMessage(), report);

		List<String> texts = runs.stream().map(r -> r.text).collect(java.util.stream.Collectors.toList());
		assertTrue(texts.stream().anyMatch(t -> t != null && t.contains("but still loses his action")));
	}

	@Test
	public void reportIdIsAnimalSavagery() {
		assertEquals(ReportId.ANIMAL_SAVAGERY.getKey(), new AnimalSavageryMessage().getKey());
	}
}
