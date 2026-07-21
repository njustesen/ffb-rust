package com.fumbbl.ffb.client.report.bb2025;

import com.fumbbl.ffb.client.report.ReportMessageTestBase;
import com.fumbbl.ffb.model.Player;
import com.fumbbl.ffb.report.ReportId;
import com.fumbbl.ffb.report.bb2025.ReportChompRemoved;
import org.junit.jupiter.api.Test;
import org.mockito.Mock;

import java.util.List;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.BDDMockito.given;

class ChompRemovedMessageTest extends ReportMessageTestBase {

	@SuppressWarnings("rawtypes")
	@Mock
	private Player player;

	@Test
	public void successfulUnchompIsFreeToMove() {
		given(game.getPlayerById("p1")).willReturn(player);
		given(game.getTeamHome().hasPlayer(player)).willReturn(true);

		ReportChompRemoved report = new ReportChompRemoved("p1", true);
		List<Run> runs = render(new ChompRemovedMessage(), report);
		List<String> texts = runs.stream().map(r -> r.text).collect(java.util.stream.Collectors.toList());
		assertTrue(texts.stream().anyMatch(t -> t != null && t.contains("is free to move again.")));
	}

	@Test
	public void unsuccessfulUnchompIsStillHeld() {
		given(game.getPlayerById("p1")).willReturn(player);
		given(game.getTeamHome().hasPlayer(player)).willReturn(true);

		ReportChompRemoved report = new ReportChompRemoved("p1", false);
		List<Run> runs = render(new ChompRemovedMessage(), report);
		List<String> texts = runs.stream().map(r -> r.text).collect(java.util.stream.Collectors.toList());
		assertTrue(texts.stream().anyMatch(t -> t != null && t.contains("but is still held by another player.")));
	}

	@Test
	public void printsPlayerName() {
		given(game.getPlayerById("p1")).willReturn(player);
		given(game.getTeamHome().hasPlayer(player)).willReturn(true);
		given(player.getName()).willReturn("Chomped Player");

		ReportChompRemoved report = new ReportChompRemoved("p1", true);
		List<Run> runs = render(new ChompRemovedMessage(), report);
		List<String> texts = runs.stream().map(r -> r.text).collect(java.util.stream.Collectors.toList());
		assertTrue(texts.stream().anyMatch(t -> "Chomped Player".equals(t)));
	}

	@Test
	public void reportIdIsChompRemoved() {
		assertEquals(ReportId.CHOMP_REMOVED.getKey(), new ChompRemovedMessage().getKey());
	}
}
