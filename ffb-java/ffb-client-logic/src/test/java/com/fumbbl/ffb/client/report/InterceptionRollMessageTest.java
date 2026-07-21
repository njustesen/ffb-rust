package com.fumbbl.ffb.client.report;

import com.fumbbl.ffb.FactoryType.Factory;
import com.fumbbl.ffb.mechanics.Mechanic;
import com.fumbbl.ffb.model.Player;
import com.fumbbl.ffb.report.ReportInterceptionRoll;
import org.junit.jupiter.api.Test;
import org.mockito.Mock;

import java.util.ArrayList;
import java.util.List;

import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertNotNull;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.BDDMockito.given;

class InterceptionRollMessageTest extends ReportMessageTestBase {

	@SuppressWarnings("rawtypes")
	@Mock
	private Player player;

	private void stubMechanic(com.fumbbl.ffb.mechanics.AgilityMechanic mechanic) {
		given(game.getFactory(Factory.MECHANIC).forName(Mechanic.Type.AGILITY.name())).willReturn(mechanic);
		given(game.getRules().getFactory(Factory.MECHANIC).forName(Mechanic.Type.AGILITY.name())).willReturn(mechanic);
	}

	private void stubPlayer(int agility) {
		given(game.getPlayerById("p1")).willReturn(player);
		given(player.getAgilityWithModifiers()).willReturn(agility);
		given(game.getTeamHome().hasPlayer(player)).willReturn(true);
	}

	private List<String> texts(List<Run> runs) {
		List<String> texts = new ArrayList<>();
		for (Run run : runs) {
			if (run.text != null) {
				texts.add(run.text);
			}
		}
		return texts;
	}

	private boolean anyContains(List<String> texts, String needle) {
		for (String text : texts) {
			if (text.contains(needle)) {
				return true;
			}
		}
		return false;
	}

	private String findContaining(List<String> texts, String needle) {
		for (String text : texts) {
			if (text.contains(needle)) {
				return text;
			}
		}
		return null;
	}

	@Test
	public void successfulBallInterceptionReportsVerbAndInflection() {
		stubPlayer(3);
		stubMechanic(new com.fumbbl.ffb.mechanics.bb2025.AgilityMechanic());

		ReportInterceptionRoll report = new ReportInterceptionRoll("p1", true, 5, 3, false, null, false, false);
		List<Run> runs = render(new InterceptionRollMessage(), report);

		List<String> texts = texts(runs);
		assertTrue(anyContains(texts, "tries to intercept the ball:"));
		assertTrue(anyContains(texts, "intercepts the ball."));
		assertTrue(anyContains(texts, "Succeeded on a roll of 3+"));
	}

	@Test
	public void failedBombInterceptionReportsFailsToIntercept() {
		stubPlayer(3);
		stubMechanic(new com.fumbbl.ffb.mechanics.bb2025.AgilityMechanic());

		ReportInterceptionRoll report = new ReportInterceptionRoll("p1", false, 1, 4, false, null, true, false);
		List<Run> runs = render(new InterceptionRollMessage(), report);

		List<String> texts = texts(runs);
		assertTrue(anyContains(texts, "tries to intercept the bomb:"));
		assertTrue(anyContains(texts, "fails to intercept the bomb."));
		assertTrue(anyContains(texts, "Roll a 4+ to succeed"));
	}

	@Test
	public void reRolledSkipsInitialAttemptLineAndNeededRoll() {
		stubPlayer(3);
		stubMechanic(new com.fumbbl.ffb.mechanics.bb2025.AgilityMechanic());

		ReportInterceptionRoll report = new ReportInterceptionRoll("p1", true, 5, 3, true, null, false, false);
		List<Run> runs = render(new InterceptionRollMessage(), report);

		List<String> texts = texts(runs);
		assertFalse(anyContains(texts, "tries to intercept"));
		assertFalse(anyContains(texts, "Succeeded on a roll of"));
	}

	@Test
	public void ignoreAgilitySkipsNeededRollFormulaSuffix() {
		stubPlayer(3);
		stubMechanic(new com.fumbbl.ffb.mechanics.bb2025.AgilityMechanic());

		ReportInterceptionRoll report = new ReportInterceptionRoll("p1", false, 1, 4, false, null, false, true);
		List<Run> runs = render(new InterceptionRollMessage(), report);

		List<String> texts = texts(runs);
		String needed = findContaining(texts, "Roll a 4+ to succeed");
		assertNotNull(needed);
		assertFalse(needed.contains("(Roll"));
	}

	@Test
	public void bb2016NeededRollUsesAgFormulaNotBb2025RollFormula() {
		// Regression test: BB2016's AgilityMechanic.formatInterceptionResult uses the
		// "(AG X - 2 Interception ... + Roll > 6)." phrasing, not BB2020/BB2025's
		// "(Roll ... >= N+)" phrasing.
		stubPlayer(3);
		stubMechanic(new com.fumbbl.ffb.mechanics.bb2016.AgilityMechanic());

		ReportInterceptionRoll report = new ReportInterceptionRoll("p1", false, 1, 4, false, null, false, false);
		List<Run> runs = render(new InterceptionRollMessage(), report);

		List<String> texts = texts(runs);
		String needed = findContaining(texts, "Roll a 4+ to succeed");
		assertNotNull(needed);
		// Note: unlike the Rust port's "Interception+ Roll" (no space), the real Java
		// AgilityMechanic.formatInterceptionResult concatenates a space before the
		// modifiers suffix, producing "Interception + Roll" (with space) when there
		// are no roll modifiers.
		assertTrue(needed.contains("(AG 3 - 2 Interception + Roll > 6)."));
		assertFalse(needed.contains("(Roll >="));
	}
}
