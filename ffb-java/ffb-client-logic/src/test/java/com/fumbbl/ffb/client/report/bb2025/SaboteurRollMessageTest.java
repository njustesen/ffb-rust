package com.fumbbl.ffb.client.report.bb2025;

import com.fumbbl.ffb.client.TextStyle;
import com.fumbbl.ffb.client.report.ReportMessageTestBase;
import com.fumbbl.ffb.model.Player;
import com.fumbbl.ffb.report.ReportId;
import com.fumbbl.ffb.report.bb2025.ReportSaboteurRoll;
import org.junit.jupiter.api.Test;
import org.mockito.Mock;

import java.util.List;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.BDDMockito.given;

class SaboteurRollMessageTest extends ReportMessageTestBase {

	@SuppressWarnings("rawtypes")
	@Mock
	private Player saboteur;

	@Test
	public void reportIdIsSaboteurRoll() {
		assertEquals(ReportId.SABOTEUR_ROLL.getKey(), new SaboteurRollMessage().getKey());
	}

	@Test
	public void successfulRollReportsSabotage() {
		given(game.getPlayerById("p1")).willReturn(saboteur);
		given(saboteur.getName()).willReturn("Saboteur");
		given(game.getTeamHome().hasPlayer(saboteur)).willReturn(true);

		ReportSaboteurRoll report = new ReportSaboteurRoll("p1", true, 4, 3, false);
		List<Run> runs = render(new SaboteurRollMessage(), report);

		assertTrue(runs.stream().anyMatch(r -> "Saboteur Roll [ 4 ]".equals(r.text)));
		assertTrue(runs.stream().anyMatch(r -> " sabotages their weapon! They are KO'd and the blocker is knocked down.".equals(r.text)));
	}

	@Test
	public void unsuccessfulRollReportsFailure() {
		given(game.getPlayerById("p1")).willReturn(saboteur);
		given(saboteur.getName()).willReturn("Saboteur");
		given(game.getTeamHome().hasPlayer(saboteur)).willReturn(true);

		ReportSaboteurRoll report = new ReportSaboteurRoll("p1", false, 2, 3, false);
		List<Run> runs = render(new SaboteurRollMessage(), report);

		assertTrue(runs.stream().anyMatch(r -> " fails to detonate the weapon.".equals(r.text)));
	}

	@Test
	public void rollTextUsesRollStyle() {
		given(game.getPlayerById("p1")).willReturn(saboteur);
		given(saboteur.getName()).willReturn("Saboteur");
		given(game.getTeamHome().hasPlayer(saboteur)).willReturn(true);

		ReportSaboteurRoll report = new ReportSaboteurRoll("p1", true, 6, 3, false);
		List<Run> runs = render(new SaboteurRollMessage(), report);

		assertEquals(TextStyle.ROLL, runs.get(0).textStyle);
	}
}
