package com.fumbbl.ffb.client.report;

import com.fumbbl.ffb.client.TextStyle;
import com.fumbbl.ffb.model.Player;
import com.fumbbl.ffb.report.ReportDefectingPlayers;
import org.junit.jupiter.api.Test;
import org.mockito.Mock;

import java.util.List;
import java.util.stream.Collectors;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.BDDMockito.given;

class DefectingPlayersMessageTest extends ReportMessageTestBase {

	@SuppressWarnings("rawtypes")
	@Mock
	private Player player1;

	@SuppressWarnings("rawtypes")
	@Mock
	private Player player2;

	@Test
	public void emptyPlayerIdsRendersNothing() {
		ReportDefectingPlayers report = new ReportDefectingPlayers(new String[0], new int[0], new boolean[0]);
		List<Run> runs = render(new DefectingPlayersMessage(), report);

		assertTrue(runs.isEmpty());
	}

	@Test
	public void defectingPlayerPrintsDisgustMessage() {
		given(game.getPlayerById("p2")).willReturn(player2);
		given(player2.getName()).willReturn("Deserter");
		given(game.getTeamHome().hasPlayer(player2)).willReturn(true);

		ReportDefectingPlayers report = new ReportDefectingPlayers(new String[] { "p2" }, new int[] { 1 },
			new boolean[] { true });
		List<Run> runs = render(new DefectingPlayersMessage(), report);

		assertTrue(runs.stream().anyMatch(r -> " leaves the team in disgust.".equals(r.text)));
	}

	@Test
	public void stayingPlayerPrintsStaysMessage() {
		given(game.getPlayerById("p1")).willReturn(player1);
		given(player1.getName()).willReturn("Loyal");
		given(game.getTeamHome().hasPlayer(player1)).willReturn(true);

		ReportDefectingPlayers report = new ReportDefectingPlayers(new String[] { "p1" }, new int[] { 5 },
			new boolean[] { false });
		List<Run> runs = render(new DefectingPlayersMessage(), report);

		assertTrue(runs.stream().anyMatch(r -> " stays with the team.".equals(r.text)));
	}

	@Test
	public void multiplePlayersRenderOneBlockEach() {
		given(game.getPlayerById("p1")).willReturn(player1);
		given(player1.getName()).willReturn("Loyal");
		given(game.getTeamHome().hasPlayer(player1)).willReturn(true);
		given(game.getPlayerById("p2")).willReturn(player2);
		given(player2.getName()).willReturn("Deserter");
		given(game.getTeamHome().hasPlayer(player2)).willReturn(true);

		ReportDefectingPlayers report = new ReportDefectingPlayers(new String[] { "p1", "p2" }, new int[] { 2, 6 },
			new boolean[] { false, true });
		List<Run> runs = render(new DefectingPlayersMessage(), report);

		long rollLines = runs.stream().filter(r -> r.textStyle == TextStyle.ROLL).count();
		assertEquals(2, rollLines);

		List<String> names = runs.stream().map(r -> r.text)
			.filter(t -> "Loyal".equals(t) || "Deserter".equals(t))
			.collect(Collectors.toList());
		assertEquals(List.of("Loyal", "Deserter"), names);
	}

	@Test
	public void reportIdIsDefectingPlayers() {
		assertEquals("defectingPlayers", new DefectingPlayersMessage().getKey());
	}
}
