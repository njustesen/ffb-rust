package com.fumbbl.ffb.client.report.bb2025;

import com.fumbbl.ffb.PlayerGender;
import com.fumbbl.ffb.client.report.ReportMessageTestBase;
import com.fumbbl.ffb.model.Player;
import com.fumbbl.ffb.report.ReportId;
import com.fumbbl.ffb.report.mixed.ReportThenIStartedBlastin;
import org.junit.jupiter.api.Test;
import org.mockito.Mock;

import java.util.List;
import java.util.Objects;
import java.util.stream.Collectors;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.BDDMockito.given;

class ThenIStartedBlastinMessageTest extends ReportMessageTestBase {

	@SuppressWarnings("rawtypes")
	@Mock
	private Player thrower;

	@SuppressWarnings("rawtypes")
	@Mock
	private Player target;

	@Test
	public void reportIdIsThenIStartedBlastin() {
		assertEquals(ReportId.THEN_I_STARTED_BLASTIN.getKey(), new ThenIStartedBlastinMessage().getKey());
	}

	@Test
	public void noTargetStartsBlastin() {
		given(game.getPlayerById("thrower")).willReturn(thrower);

		ReportThenIStartedBlastin report = new ReportThenIStartedBlastin("thrower", null, 4, false, false);
		List<Run> runs = render(new ThenIStartedBlastinMessage(), report);
		List<String> texts = runs.stream().map(r -> r.text).filter(Objects::nonNull).collect(Collectors.toList());

		assertTrue(texts.stream().anyMatch(t -> t.equals(" starts blastin' ")));
		assertTrue(texts.stream().anyMatch(t -> t.equals("\"Blastin' Solves Everything\" Roll [ 4 ]")));
	}

	@Test
	public void fumbleHitsSelf() {
		given(game.getPlayerById("thrower")).willReturn(thrower);
		given(thrower.getPlayerGender()).willReturn(PlayerGender.MALE);

		ReportThenIStartedBlastin report = new ReportThenIStartedBlastin("thrower", "target", 1, false, true);
		List<Run> runs = render(new ThenIStartedBlastinMessage(), report);
		List<String> texts = runs.stream().map(r -> r.text).filter(Objects::nonNull).collect(Collectors.toList());

		assertTrue(texts.stream().anyMatch(t -> t.equals("himself")));
	}

	@Test
	public void successHitsTargetPlayer() {
		given(game.getPlayerById("thrower")).willReturn(thrower);
		given(game.getPlayerById("target")).willReturn(target);
		given(target.getName()).willReturn("Target");

		ReportThenIStartedBlastin report = new ReportThenIStartedBlastin("thrower", "target", 5, true, false);
		List<Run> runs = render(new ThenIStartedBlastinMessage(), report);
		List<String> texts = runs.stream().map(r -> r.text).filter(Objects::nonNull).collect(Collectors.toList());

		assertTrue(texts.stream().anyMatch(t -> t.equals("Target")));
	}

	@Test
	public void missHitsAPlayerChosenByOpposingCoach() {
		given(game.getPlayerById("thrower")).willReturn(thrower);
		given(game.getPlayerById("target")).willReturn(target);

		ReportThenIStartedBlastin report = new ReportThenIStartedBlastin("thrower", "target", 2, false, false);
		List<Run> runs = render(new ThenIStartedBlastinMessage(), report);
		List<String> texts = runs.stream().map(r -> r.text).filter(Objects::nonNull).collect(Collectors.toList());

		assertTrue(texts.stream().anyMatch(t -> t.equals("a player chosen by the opposing coach")));
	}

	@Test
	public void rollZeroSkipsRollLine() {
		given(game.getPlayerById("thrower")).willReturn(thrower);

		ReportThenIStartedBlastin report = new ReportThenIStartedBlastin("thrower", null, 0, false, false);
		List<Run> runs = render(new ThenIStartedBlastinMessage(), report);
		List<String> texts = runs.stream().map(r -> r.text).filter(Objects::nonNull).collect(Collectors.toList());

		assertFalse(texts.stream().anyMatch(t -> t.contains("Blastin'")));
	}
}
