package com.fumbbl.ffb.client.report.bb2025;

import com.fumbbl.ffb.PlayerGender;
import com.fumbbl.ffb.client.report.ReportMessageTestBase;
import com.fumbbl.ffb.model.Player;
import com.fumbbl.ffb.report.ReportId;
import com.fumbbl.ffb.report.bb2025.ReportSteadyFootingRoll;
import org.junit.jupiter.api.Test;
import org.mockito.Mock;

import java.util.List;
import java.util.Objects;
import java.util.stream.Collectors;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.mockito.BDDMockito.given;

class SteadyFootingRollMessageTest extends ReportMessageTestBase {

	@Mock
	@SuppressWarnings("rawtypes")
	private Player player;

	@Test
	public void successfulRollNotRerolledShowsNeededRoll() {
		given(game.getPlayerById("p1")).willReturn(player);
		given(player.getPlayerGender()).willReturn(PlayerGender.MALE);

		ReportSteadyFootingRoll report = new ReportSteadyFootingRoll("p1", true, 4, 3, false);
		List<Run> runs = render(new SteadyFootingRollMessage(), report);
		List<String> texts = runs.stream().map(r -> r.text).collect(Collectors.toList());
		assertEquals(true, texts.stream().anyMatch(t -> "Steady Footing Roll [ 4 ]".equals(t)));
		assertEquals(true, texts.stream().anyMatch(t -> " stays on his  feet.".equals(t)));
		assertEquals(true, texts.stream().anyMatch(t -> "Succeeded on a roll of 3+".equals(t)));
	}

	@Test
	public void failedRollRerolledHidesNeededRoll() {
		given(game.getPlayerById("p1")).willReturn(player);
		given(player.getPlayerGender()).willReturn(PlayerGender.MALE);

		ReportSteadyFootingRoll report = new ReportSteadyFootingRoll("p1", false, 2, 3, true);
		List<Run> runs = render(new SteadyFootingRollMessage(), report);
		List<String> texts = runs.stream().map(r -> r.text).collect(Collectors.toList());
		assertEquals(true, texts.stream().anyMatch(t -> " fails to keep standing.".equals(t)));
		assertEquals(false, texts.stream().filter(Objects::nonNull).anyMatch(t -> t.contains("Roll a")));
	}

	@Test
	public void zeroRollUsesNewResultHeader() {
		given(game.getPlayerById("p1")).willReturn(player);
		given(player.getPlayerGender()).willReturn(PlayerGender.MALE);

		ReportSteadyFootingRoll report = new ReportSteadyFootingRoll("p1", false, 0, 3, false);
		List<Run> runs = render(new SteadyFootingRollMessage(), report);
		List<String> texts = runs.stream().map(r -> r.text).collect(Collectors.toList());
		assertEquals(true, texts.stream().anyMatch(t -> "New Steady Footing Result".equals(t)));
		assertEquals(true, texts.stream().anyMatch(t -> "Roll a 3+ to succeed".equals(t)));
	}

	@Test
	public void reportIdIsSteadyFootingRoll() {
		assertEquals(ReportId.STEADY_FOOTING_ROLL.getKey(), new SteadyFootingRollMessage().getKey());
	}
}
