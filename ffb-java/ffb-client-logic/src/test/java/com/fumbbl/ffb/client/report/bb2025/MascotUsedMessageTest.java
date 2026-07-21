package com.fumbbl.ffb.client.report.bb2025;

import com.fumbbl.ffb.client.report.ReportMessageTestBase;
import com.fumbbl.ffb.report.ReportId;
import com.fumbbl.ffb.report.bb2025.ReportMascotUsed;
import org.junit.jupiter.api.Test;

import java.util.List;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.BDDMockito.given;

class MascotUsedMessageTest extends ReportMessageTestBase {

	private void stubTeams() {
		given(game.getTeamHome().getId()).willReturn("home");
		given(game.getTeamHome().getName()).willReturn("Team home");
		given(game.getTeamAway().getId()).willReturn("away");
		given(game.getTeamAway().getName()).willReturn("Team away");
	}

	@Test
	public void successfulMascotUse() {
		stubTeams();
		ReportMascotUsed report = new ReportMascotUsed("home", 4, 5, true, false);
		List<Run> runs = render(new MascotUsedMessage(), report);
		List<String> texts = runs.stream().map(r -> r.text).collect(java.util.stream.Collectors.toList());
		assertTrue(texts.stream().anyMatch(t -> t != null && t.contains("used their Team Mascot successfully.")));
		assertFalse(texts.stream().anyMatch(t -> t != null && t.contains("Roll >=")));
	}

	@Test
	public void failedWithFallbackReroll() {
		stubTeams();
		ReportMascotUsed report = new ReportMascotUsed("home", 4, 2, false, true);
		List<Run> runs = render(new MascotUsedMessage(), report);
		List<String> texts = runs.stream().map(r -> r.text).collect(java.util.stream.Collectors.toList());
		assertTrue(texts.stream().anyMatch(t -> t != null && t.contains("but it failed so they used a regular re-roll instead.")));
		assertTrue(texts.stream().anyMatch(t -> t != null && t.contains("Roll >= 4 to succeed")));
	}

	@Test
	public void failedWithoutFallback() {
		stubTeams();
		ReportMascotUsed report = new ReportMascotUsed("away", 5, 1, false, false);
		List<Run> runs = render(new MascotUsedMessage(), report);
		List<String> texts = runs.stream().map(r -> r.text).collect(java.util.stream.Collectors.toList());
		assertTrue(texts.stream().anyMatch(t -> " used their Team Mascot but it failed.".equals(t)));
	}

	@Test
	public void reportIdIsMascotUsed() {
		assertEquals(ReportId.MASCOT_USED.getKey(), new MascotUsedMessage().getKey());
	}
}
