package com.fumbbl.ffb.client.report.bb2025;

import com.fumbbl.ffb.PassingDistance;
import com.fumbbl.ffb.client.report.ReportMessageTestBase;
import com.fumbbl.ffb.mechanics.PassResult;
import com.fumbbl.ffb.model.Player;
import com.fumbbl.ffb.modifiers.PassModifier;
import com.fumbbl.ffb.report.mixed.ReportThrowTeamMateRoll;
import org.junit.jupiter.api.Test;
import org.mockito.Mock;

import java.util.List;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.BDDMockito.given;

class ThrowTeamMateRollMessageTest extends ReportMessageTestBase {

	@SuppressWarnings("rawtypes")
	@Mock
	private Player thrower;

	@SuppressWarnings("rawtypes")
	@Mock
	private Player thrownPlayer;

	private void stubPlayers() {
		given(game.getActingPlayer().getPlayer()).willReturn(thrower);
		given(game.getPlayerById("thrown")).willReturn(thrownPlayer);
		given(thrower.getName()).willReturn("Thrower");
		given(thrower.getPassing()).willReturn(3);
		given(thrownPlayer.getName()).willReturn("Thrown");
		given(game.getTeamHome().hasPlayer(thrower)).willReturn(true);
		given(game.getTeamHome().hasPlayer(thrownPlayer)).willReturn(false);
	}

	@Test
	public void getKeyIsThrowTeamMateRoll() {
		assertEquals("throwTeamMateRoll", new ThrowTeamMateRollMessage().getKey());
	}

	@Test
	public void successfulAccurateThrowReportsSuperbly() {
		stubPlayers();
		ReportThrowTeamMateRoll report = new ReportThrowTeamMateRoll(
			"thrower", true, 4, 3, false, new PassModifier[0], PassingDistance.SHORT_PASS, "thrown", PassResult.ACCURATE, false);
		List<Run> runs = render(new ThrowTeamMateRollMessage(), report);

		assertTrue(runs.stream().anyMatch(r -> "Throw Team-Mate Roll [ 4 ]".equals(r.text)));
		assertTrue(runs.stream().anyMatch(r -> " superbly.".equals(r.text)));
		assertTrue(runs.stream().anyMatch(r -> r.text != null && r.text.contains("Succeeded on a roll of 3+ to avoid a Fumbled Throw")));
	}

	@Test
	public void kickUsesKickWording() {
		stubPlayers();
		ReportThrowTeamMateRoll report = new ReportThrowTeamMateRoll(
			"thrower", true, 4, 3, false, new PassModifier[0], PassingDistance.SHORT_PASS, "thrown", PassResult.INACCURATE, true);
		List<Run> runs = render(new ThrowTeamMateRollMessage(), report);

		assertTrue(runs.stream().anyMatch(r -> "Kick Team-Mate Roll [ 4 ]".equals(r.text)));
		assertTrue(runs.stream().anyMatch(r -> " with a subpar result.".equals(r.text)));
		assertTrue(runs.stream().anyMatch(r -> r.text != null && r.text.contains("to avoid a Fumbled Kick")));
	}

	@Test
	public void fumbleReportsFumbles() {
		stubPlayers();
		ReportThrowTeamMateRoll report = new ReportThrowTeamMateRoll(
			"thrower", false, 1, 3, false, new PassModifier[0], PassingDistance.SHORT_PASS, "thrown", null, false);
		List<Run> runs = render(new ThrowTeamMateRollMessage(), report);

		assertTrue(runs.stream().anyMatch(r -> r.text != null && r.text.contains(" fumbles ")));
		assertTrue(runs.stream().anyMatch(r -> r.text != null && r.text.contains("Roll a 3+ to make at least a Subpar Throw")));
	}

	@Test
	public void reRolledSkipsIntroAndNeededRoll() {
		stubPlayers();
		ReportThrowTeamMateRoll report = new ReportThrowTeamMateRoll(
			"thrower", true, 4, 3, true, new PassModifier[0], PassingDistance.SHORT_PASS, "thrown", PassResult.ACCURATE, false);
		List<Run> runs = render(new ThrowTeamMateRollMessage(), report);

		assertFalse(runs.stream().anyMatch(r -> r.text != null && r.text.contains("tries to throw")));
		assertFalse(runs.stream().anyMatch(r -> r.text != null && r.text.contains("Succeeded on a roll of")));
	}

	@Test
	public void indentIncrementedAfterRender() {
		stubPlayers();
		ReportThrowTeamMateRoll report = new ReportThrowTeamMateRoll(
			"thrower", true, 4, 3, false, new PassModifier[0], PassingDistance.SHORT_PASS, "thrown", PassResult.ACCURATE, false);
		int before = statusReport.getIndent();
		render(new ThrowTeamMateRollMessage(), report);
		assertEquals(before + 1, statusReport.getIndent());
	}
}
