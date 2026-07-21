package com.fumbbl.ffb.client.report;

import com.fumbbl.ffb.PlayerGender;
import com.fumbbl.ffb.model.Player;
import com.fumbbl.ffb.report.ReportWeepingDaggerRoll;
import org.junit.jupiter.api.Test;
import org.mockito.Mock;

import java.util.List;
import java.util.stream.Collectors;

import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.BDDMockito.given;

class WeepingDaggerRollMessageTest extends ReportMessageTestBase {

	@SuppressWarnings("rawtypes")
	@Mock
	private Player player;

	@Test
	public void successfulRollPoisonsOpponent() {
		given(game.getPlayerById("p1")).willReturn(player);
		given(player.getName()).willReturn("Snik");
		given(player.getPlayerGender()).willReturn(PlayerGender.MALE);
		given(game.getTeamHome().hasPlayer(player)).willReturn(true);

		ReportWeepingDaggerRoll report = new ReportWeepingDaggerRoll("p1", true, 5, 2, false, null);
		List<Run> runs = render(new WeepingDaggerRollMessage(), report);

		List<String> texts = runs.stream().map(r -> r.text).collect(Collectors.toList());
		assertTrue(texts.contains("Weeping Dagger Roll [ 5 ]"));
		assertTrue(texts.contains(" poisons his opponent."));
	}

	@Test
	public void failedRollFailsToPoison() {
		given(game.getPlayerById("p1")).willReturn(player);
		given(player.getName()).willReturn("Snik");
		given(player.getPlayerGender()).willReturn(PlayerGender.FEMALE);
		given(game.getTeamHome().hasPlayer(player)).willReturn(true);

		ReportWeepingDaggerRoll report = new ReportWeepingDaggerRoll("p1", false, 1, 2, false, null);
		List<Run> runs = render(new WeepingDaggerRollMessage(), report);

		List<String> texts = runs.stream().map(r -> r.text).collect(Collectors.toList());
		assertTrue(texts.contains(" fails to poison her opponent."));
	}

	@Test
	public void nonbinaryGenderUsesTheir() {
		given(game.getPlayerById("p1")).willReturn(player);
		given(player.getName()).willReturn("Snik");
		given(player.getPlayerGender()).willReturn(PlayerGender.NONBINARY);
		given(game.getTeamHome().hasPlayer(player)).willReturn(true);

		ReportWeepingDaggerRoll report = new ReportWeepingDaggerRoll("p1", true, 5, 2, false, null);
		List<Run> runs = render(new WeepingDaggerRollMessage(), report);

		List<String> texts = runs.stream().map(r -> r.text).collect(Collectors.toList());
		assertTrue(texts.contains(" poisons their opponent."));
	}
}
