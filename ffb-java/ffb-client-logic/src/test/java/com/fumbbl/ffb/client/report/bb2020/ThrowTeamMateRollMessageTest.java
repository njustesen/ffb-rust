package com.fumbbl.ffb.client.report.bb2020;

import com.fumbbl.ffb.PassingDistance;
import com.fumbbl.ffb.client.report.ReportMessageTestBase;
import com.fumbbl.ffb.mechanics.PassResult;
import com.fumbbl.ffb.model.ActingPlayer;
import com.fumbbl.ffb.model.Player;
import com.fumbbl.ffb.modifiers.PassModifier;
import com.fumbbl.ffb.report.mixed.ReportThrowTeamMateRoll;
import org.junit.jupiter.api.Test;
import org.mockito.Mock;

import java.util.List;

import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.BDDMockito.given;

class ThrowTeamMateRollMessageTest extends ReportMessageTestBase {

	@SuppressWarnings("rawtypes")
	@Mock
	private Player thrower;

	@SuppressWarnings("rawtypes")
	@Mock
	private Player thrownPlayer;

	@Mock
	private ActingPlayer actingPlayer;

	private void stubThrower(int passing) {
		given(game.getActingPlayer()).willReturn(actingPlayer);
		given(actingPlayer.getPlayer()).willReturn(thrower);
		given(thrower.getPassing()).willReturn(passing);
		given(game.getPlayerById("thrown")).willReturn(thrownPlayer);
	}

	@Test
	public void kickSuccessReportsSuperblyAndBumpsIndent() {
		stubThrower(3);
		ReportThrowTeamMateRoll report = new ReportThrowTeamMateRoll(
			"thrower", true, 6, 2, false, new PassModifier[0], PassingDistance.SHORT_PASS,
			"thrown", PassResult.ACCURATE, true);
		int startIndent = statusReport.getIndent();
		List<Run> runs = render(new ThrowTeamMateRollMessage(), report);
		assertTrue(runs.stream().anyMatch(r -> " kicks ".equals(r.text)));
		assertTrue(runs.stream().anyMatch(r -> r.text != null && r.text.contains("superbly")));
		assertTrue(statusReport.getIndent() == startIndent + 1);
	}

	@Test
	public void throwWildlyInaccurateReportsDeviate() {
		stubThrower(3);
		ReportThrowTeamMateRoll report = new ReportThrowTeamMateRoll(
			"thrower", false, 1, 2, false, new PassModifier[0], PassingDistance.SHORT_PASS,
			"thrown", PassResult.WILDLY_INACCURATE, false);
		List<Run> runs = render(new ThrowTeamMateRollMessage(), report);
		assertTrue(runs.stream().anyMatch(r -> " lets ".equals(r.text)));
		assertTrue(runs.stream().anyMatch(r -> " deviate.".equals(r.text)));
	}

	@Test
	public void throwFumbleReportsFumbles() {
		stubThrower(3);
		ReportThrowTeamMateRoll report = new ReportThrowTeamMateRoll(
			"thrower", false, 1, 2, false, new PassModifier[0], PassingDistance.SHORT_PASS,
			"thrown", PassResult.FUMBLE, false);
		List<Run> runs = render(new ThrowTeamMateRollMessage(), report);
		assertTrue(runs.stream().anyMatch(r -> " fumbles ".equals(r.text)));
	}

	@Test
	public void reRolledSkipsIntroLine() {
		stubThrower(3);
		ReportThrowTeamMateRoll report = new ReportThrowTeamMateRoll(
			"thrower", true, 6, 2, true, new PassModifier[0], PassingDistance.SHORT_PASS,
			"thrown", PassResult.ACCURATE, false);
		List<Run> runs = render(new ThrowTeamMateRollMessage(), report);
		assertTrue(runs.stream().noneMatch(r -> r.text != null && r.text.contains("tries to throw")));
		// re-rolled -> no "needed roll" line either.
		assertTrue(runs.stream().noneMatch(r -> r.text != null && r.text.contains("Succeeded on a roll of")));
	}

	@Test
	public void successNeededRollLineIncludedWhenNotRerolledAndCanThrow() {
		stubThrower(3);
		ReportThrowTeamMateRoll report = new ReportThrowTeamMateRoll(
			"thrower", true, 6, 2, false, new PassModifier[0], PassingDistance.SHORT_PASS,
			"thrown", PassResult.ACCURATE, false);
		List<Run> runs = render(new ThrowTeamMateRollMessage(), report);
		assertTrue(runs.stream().anyMatch(r -> r.text != null && r.text.contains("Succeeded on a roll of 2")));
		assertTrue(runs.stream().anyMatch(r -> r.text != null && r.text.contains("to avoid a fumble or terrible throw")));
	}
}
