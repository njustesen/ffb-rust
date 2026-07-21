package com.fumbbl.ffb.client.report.mixed;

import com.fumbbl.ffb.Direction;
import com.fumbbl.ffb.client.report.ReportMessageTestBase;
import com.fumbbl.ffb.model.Player;
import com.fumbbl.ffb.report.mixed.ReportPlaceBallDirection;
import org.junit.jupiter.api.Test;
import org.mockito.Mock;

import java.util.List;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.mockito.BDDMockito.given;

class PlaceBallDirectionMessageTest extends ReportMessageTestBase {

	@SuppressWarnings("rawtypes")
	@Mock
	private Player player;

	// java: `missing_direction_renders_empty_name` from the Rust suite is not portable —
	// PlaceBallDirectionMessage.render() calls `mapToLocal(report.getDirection()).getName()`
	// unconditionally, and a null direction NPEs in real Java (the Rust version guards this
	// defensively, which is a documented divergence). Skipped.

	@Test
	public void northDirection() {
		given(game.getPlayerById("p1")).willReturn(player);
		given(player.getName()).willReturn("Grobnik");
		given(game.getTeamHome().hasPlayer(player)).willReturn(true);
		given(client.getUserInterface().getPitchDimensionProvider().mapToLocal(Direction.NORTH)).willReturn(Direction.NORTH);

		ReportPlaceBallDirection report = new ReportPlaceBallDirection("p1", Direction.NORTH);
		List<Run> runs = render(new PlaceBallDirectionMessage(), report);

		assertEquals("Grobnik", runs.get(0).text);
		assertEquals(" places the ball North.", runs.get(1).text);
	}

	@Test
	public void southeastDirection() {
		given(game.getPlayerById("p1")).willReturn(player);
		given(player.getName()).willReturn("Grobnik");
		given(game.getTeamHome().hasPlayer(player)).willReturn(true);
		given(client.getUserInterface().getPitchDimensionProvider().mapToLocal(Direction.SOUTHEAST)).willReturn(Direction.SOUTHEAST);

		ReportPlaceBallDirection report = new ReportPlaceBallDirection("p1", Direction.SOUTHEAST);
		List<Run> runs = render(new PlaceBallDirectionMessage(), report);

		assertEquals(" places the ball Southeast.", runs.get(1).text);
	}
}
