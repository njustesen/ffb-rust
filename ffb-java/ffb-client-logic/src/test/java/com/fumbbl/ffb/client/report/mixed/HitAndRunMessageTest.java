package com.fumbbl.ffb.client.report.mixed;

import com.fumbbl.ffb.Direction;
import com.fumbbl.ffb.client.report.ReportMessageTestBase;
import com.fumbbl.ffb.model.Player;
import com.fumbbl.ffb.report.mixed.ReportHitAndRun;
import org.junit.jupiter.api.Test;
import org.mockito.Mock;

import java.util.List;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.mockito.BDDMockito.given;

class HitAndRunMessageTest extends ReportMessageTestBase {

	@SuppressWarnings("rawtypes")
	@Mock
	private Player player;

	@Test
	public void rendersPlayerAndDirection() {
		given(game.getPlayerById("p1")).willReturn(player);
		given(player.getName()).willReturn("Grubber");
		given(game.getTeamHome().hasPlayer(player)).willReturn(true);
		given(client.getUserInterface().getPitchDimensionProvider().mapToLocal(Direction.NORTH))
			.willReturn(Direction.NORTH);

		ReportHitAndRun report = new ReportHitAndRun("p1", Direction.NORTH);
		List<Run> runs = render(new HitAndRunMessage(), report);

		assertEquals("Grubber", runs.get(0).text);
		assertEquals(" moves one square ", runs.get(1).text);
		assertEquals("North", runs.get(2).text);
		assertEquals(".", runs.get(3).text);
	}

	@Test
	public void rendersDifferentDirection() {
		given(game.getPlayerById("p1")).willReturn(player);
		given(player.getName()).willReturn("Grubber");
		given(game.getTeamHome().hasPlayer(player)).willReturn(true);
		given(client.getUserInterface().getPitchDimensionProvider().mapToLocal(Direction.SOUTHEAST))
			.willReturn(Direction.SOUTHEAST);

		ReportHitAndRun report = new ReportHitAndRun("p1", Direction.SOUTHEAST);
		List<Run> runs = render(new HitAndRunMessage(), report);

		assertEquals("Southeast", runs.get(2).text);
	}

	@Test
	public void skipsPlayerRunWhenPlayerNotFound() {
		given(game.getPlayerById("missing")).willReturn(null);
		given(client.getUserInterface().getPitchDimensionProvider().mapToLocal(Direction.WEST))
			.willReturn(Direction.WEST);

		ReportHitAndRun report = new ReportHitAndRun("missing", Direction.WEST);
		List<Run> runs = render(new HitAndRunMessage(), report);

		// print(indent, false, player) emits nothing for an unresolved player.
		assertEquals(" moves one square ", runs.get(0).text);
		assertEquals("West", runs.get(1).text);
	}
}
