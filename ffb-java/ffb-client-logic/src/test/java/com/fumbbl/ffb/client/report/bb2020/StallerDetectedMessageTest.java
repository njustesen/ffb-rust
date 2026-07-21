package com.fumbbl.ffb.client.report.bb2020;

import com.fumbbl.ffb.client.report.ReportMessageTestBase;
import com.fumbbl.ffb.model.Player;
import com.fumbbl.ffb.report.mixed.ReportStallerDetected;
import org.junit.jupiter.api.Test;
import org.mockito.Mock;

import java.util.List;

import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.BDDMockito.given;

class StallerDetectedMessageTest extends ReportMessageTestBase {

	@SuppressWarnings("rawtypes")
	@Mock
	private Player player;

	@Test
	public void headerIsAlwaysPrinted() {
		ReportStallerDetected report = new ReportStallerDetected(null);
		List<Run> runs = render(new StallerDetectedMessage(), report);
		List<String> texts = runs.stream().map(r -> r.text).collect(java.util.stream.Collectors.toList());

		assertTrue(texts.contains("Stalling Detection"));
	}

	@Test
	public void playerPresentPrintsPlayerName() {
		given(game.getPlayerById("p1")).willReturn(player);
		given(player.getName()).willReturn("Player p1");
		given(game.getTeamHome().hasPlayer(player)).willReturn(true);

		ReportStallerDetected report = new ReportStallerDetected("p1");
		List<Run> runs = render(new StallerDetectedMessage(), report);
		List<String> texts = runs.stream().map(r -> r.text).collect(java.util.stream.Collectors.toList());

		assertTrue(texts.contains("Player p1"));
		assertTrue(texts.stream().anyMatch(t -> t != null && t.contains(" is stalling")));
	}

	@Test
	public void noPlayerPrintsNobody() {
		ReportStallerDetected report = new ReportStallerDetected(null);
		List<Run> runs = render(new StallerDetectedMessage(), report);
		List<String> texts = runs.stream().map(r -> r.text).collect(java.util.stream.Collectors.toList());

		assertTrue(texts.contains("Nobody"));
	}

	@Test
	public void emptyPlayerIdTreatedAsAbsent() {
		ReportStallerDetected report = new ReportStallerDetected("");
		List<Run> runs = render(new StallerDetectedMessage(), report);
		List<String> texts = runs.stream().map(r -> r.text).collect(java.util.stream.Collectors.toList());

		assertTrue(texts.contains("Nobody"));
	}
}
