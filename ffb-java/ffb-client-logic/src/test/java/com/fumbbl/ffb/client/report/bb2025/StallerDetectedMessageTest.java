package com.fumbbl.ffb.client.report.bb2025;

import com.fumbbl.ffb.client.TextStyle;
import com.fumbbl.ffb.client.report.ReportMessageTestBase;
import com.fumbbl.ffb.model.Player;
import com.fumbbl.ffb.report.ReportId;
import com.fumbbl.ffb.report.mixed.ReportStallerDetected;
import org.junit.jupiter.api.Test;
import org.mockito.Mock;

import java.util.List;
import java.util.stream.Collectors;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.mockito.BDDMockito.given;

class StallerDetectedMessageTest extends ReportMessageTestBase {

	@Mock
	@SuppressWarnings("rawtypes")
	private Player staller;

	@Test
	public void reportsStallingDetectionHeader() {
		given(game.getPlayerById("p1")).willReturn(staller);
		given(staller.getName()).willReturn("Staller");
		given(game.getTeamHome().hasPlayer(staller)).willReturn(true);

		ReportStallerDetected report = new ReportStallerDetected("p1");
		List<Run> runs = render(new StallerDetectedMessage(), report);
		List<String> texts = runs.stream().map(r -> r.text).collect(Collectors.toList());
		assertEquals(true, texts.stream().anyMatch(t -> "Stalling Detection".equals(t)));
		assertEquals(true, texts.stream().anyMatch(t -> " could stall".equals(t)));
	}

	@Test
	public void playerPrintedBold() {
		given(game.getPlayerById("p1")).willReturn(staller);
		given(staller.getName()).willReturn("Staller");
		given(game.getTeamHome().hasPlayer(staller)).willReturn(true);

		ReportStallerDetected report = new ReportStallerDetected("p1");
		List<Run> runs = render(new StallerDetectedMessage(), report);
		Run playerRun = runs.stream().filter(r -> "Staller".equals(r.text)).findFirst().orElseThrow();
		assertEquals(TextStyle.HOME_BOLD, playerRun.textStyle);
	}

	@Test
	public void missingPlayerIdPrintsNoPlayerRun() {
		given(game.getPlayerById((String) null)).willReturn(null);

		ReportStallerDetected report = new ReportStallerDetected(null);
		List<Run> runs = render(new StallerDetectedMessage(), report);
		assertEquals(false, runs.stream().anyMatch(r -> "Staller".equals(r.text)));
	}

	@Test
	public void reportIdIsStallerDetected() {
		assertEquals(ReportId.STALLER_DETECTED.getKey(), new StallerDetectedMessage().getKey());
	}
}
