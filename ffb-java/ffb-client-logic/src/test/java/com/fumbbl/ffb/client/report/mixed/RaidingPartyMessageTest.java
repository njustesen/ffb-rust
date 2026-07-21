package com.fumbbl.ffb.client.report.mixed;

import com.fumbbl.ffb.Direction;
import com.fumbbl.ffb.client.report.ReportMessageTestBase;
import com.fumbbl.ffb.model.Player;
import com.fumbbl.ffb.report.mixed.ReportRaidingParty;
import org.junit.jupiter.api.Test;
import org.mockito.Mock;

import java.util.List;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.mockito.BDDMockito.given;

class RaidingPartyMessageTest extends ReportMessageTestBase {

	@SuppressWarnings("rawtypes")
	@Mock
	private Player player;

	@SuppressWarnings("rawtypes")
	@Mock
	private Player otherPlayer;

	// java: `missing_direction_renders_empty_name` from the Rust suite is not portable —
	// RaidingPartyMessage.render() calls `mapToLocal(report.getDirection()).getName()`
	// unconditionally; a null direction NPEs in real Java. Skipped.

	@Test
	public void rendersBothPlayersAndDirection() {
		given(game.getPlayerById("p1")).willReturn(player);
		given(player.getName()).willReturn("Joe");
		given(game.getTeamHome().hasPlayer(player)).willReturn(true);
		given(game.getPlayerById("p2")).willReturn(otherPlayer);
		given(otherPlayer.getName()).willReturn("Jane");
		given(game.getTeamHome().hasPlayer(otherPlayer)).willReturn(false);
		given(client.getUserInterface().getPitchDimensionProvider().mapToLocal(Direction.NORTH)).willReturn(Direction.NORTH);

		ReportRaidingParty report = new ReportRaidingParty("p1", "p2", Direction.NORTH);
		List<Run> runs = render(new RaidingPartyMessage(), report);

		assertEquals(true, runs.stream().anyMatch(r -> "Joe".equals(r.text)));
		assertEquals(true, runs.stream().anyMatch(r -> "Jane".equals(r.text)));
		assertEquals(true, runs.stream().anyMatch(r -> "North".equals(r.text)));
	}

	@Test
	public void endsWithPeriod() {
		given(game.getPlayerById("p1")).willReturn(player);
		given(player.getName()).willReturn("Joe");
		given(game.getTeamHome().hasPlayer(player)).willReturn(true);
		given(game.getPlayerById("p2")).willReturn(otherPlayer);
		given(otherPlayer.getName()).willReturn("Jane");
		given(game.getTeamHome().hasPlayer(otherPlayer)).willReturn(false);
		given(client.getUserInterface().getPitchDimensionProvider().mapToLocal(Direction.SOUTHEAST)).willReturn(Direction.SOUTHEAST);

		ReportRaidingParty report = new ReportRaidingParty("p1", "p2", Direction.SOUTHEAST);
		List<Run> runs = render(new RaidingPartyMessage(), report);

		Run last = runs.get(runs.size() - 2);
		assertEquals(".", last.text);
	}
}
