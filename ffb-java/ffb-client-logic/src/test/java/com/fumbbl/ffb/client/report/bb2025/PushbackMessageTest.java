package com.fumbbl.ffb.client.report.bb2025;

import com.fumbbl.ffb.PlayerGender;
import com.fumbbl.ffb.PushbackMode;
import com.fumbbl.ffb.client.TextStyle;
import com.fumbbl.ffb.client.report.ReportMessageTestBase;
import com.fumbbl.ffb.model.Player;
import com.fumbbl.ffb.report.ReportId;
import com.fumbbl.ffb.report.ReportPushback;
import org.junit.jupiter.api.Test;
import org.mockito.Mock;

import java.util.List;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.BDDMockito.given;

class PushbackMessageTest extends ReportMessageTestBase {

	@SuppressWarnings("rawtypes")
	@Mock
	private Player attacker;

	@SuppressWarnings("rawtypes")
	@Mock
	private Player defender;

	@Test
	public void sideStepPrintsDefenderAvoidingPush() {
		given(game.getPlayerById("d1")).willReturn(defender);
		given(defender.getName()).willReturn("Defender");
		given(game.getTeamHome().hasPlayer(defender)).willReturn(false);

		ReportPushback report = new ReportPushback("d1", PushbackMode.SIDE_STEP);
		List<Run> runs = render(new PushbackMessage(), report);

		assertTrue(runs.stream().anyMatch(r -> "Defender".equals(r.text)));
		assertTrue(runs.stream().anyMatch(r -> " uses Sidestep to avoid being pushed.".equals(r.text)));
	}

	@Test
	public void grabPrintsActingPlayerPlacingOpponent() {
		given(game.getActingPlayer().getPlayer()).willReturn(attacker);
		given(attacker.getName()).willReturn("Attacker");
		given(attacker.getPlayerGender()).willReturn(PlayerGender.MALE);
		given(game.getTeamHome().hasPlayer(attacker)).willReturn(true);

		ReportPushback report = new ReportPushback("d1", PushbackMode.GRAB);
		List<Run> runs = render(new PushbackMessage(), report);

		assertTrue(runs.stream().anyMatch(r -> "Attacker".equals(r.text)));
		assertTrue(runs.stream().anyMatch(r -> r.text != null && r.text.contains("uses Grab to place his opponent.")));
	}

	@Test
	public void regularModePrintsNothing() {
		ReportPushback report = new ReportPushback("d1", PushbackMode.REGULAR);
		List<Run> runs = render(new PushbackMessage(), report);

		assertTrue(runs.isEmpty());
	}

	@Test
	public void sideStepUsesIndentPlusOne() {
		statusReport.setIndent(2);
		given(game.getPlayerById("d1")).willReturn(defender);
		given(defender.getName()).willReturn("Defender");
		given(game.getTeamHome().hasPlayer(defender)).willReturn(false);

		ReportPushback report = new ReportPushback("d1", PushbackMode.SIDE_STEP);
		List<Run> runs = render(new PushbackMessage(), report);

		Run run = runs.stream().filter(r -> "Defender".equals(r.text)).findFirst().orElseThrow();
		assertEquals(TextStyle.AWAY, run.textStyle);
	}

	@Test
	public void reportIdIsPushback() {
		assertEquals(ReportId.PUSHBACK.getKey(), new PushbackMessage().getKey());
	}
}
