package com.fumbbl.ffb.client.report;

import com.fumbbl.ffb.Direction;
import com.fumbbl.ffb.FieldCoordinate;
import com.fumbbl.ffb.report.ReportKickoffScatter;
import org.junit.jupiter.api.Test;

import java.util.List;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.BDDMockito.given;

class KickoffScatterMessageTest extends ReportMessageTestBase {

	private void stubDirection(Direction direction) {
		given(client.getUserInterface().getPitchDimensionProvider().mapToLocal(direction)).willReturn(direction);
	}

	@Test
	public void singleSquareUsesSingularWording() {
		stubDirection(Direction.NORTH);
		ReportKickoffScatter report = new ReportKickoffScatter(new FieldCoordinate(0, 0), Direction.NORTH, 3, 1);
		List<Run> runs = render(new KickoffScatterMessage(), report);

		assertTrue(runs.stream().anyMatch(r -> "The kick will land 1 square north of where it was aimed.".equals(r.text)));
	}

	@Test
	public void multipleSquaresUsesPluralWording() {
		stubDirection(Direction.SOUTH);
		ReportKickoffScatter report = new ReportKickoffScatter(new FieldCoordinate(0, 0), Direction.SOUTH, 5, 3);
		List<Run> runs = render(new KickoffScatterMessage(), report);

		assertTrue(runs.stream().anyMatch(r -> "The kick will land 3 squares south of where it was aimed.".equals(r.text)));
	}

	@Test
	public void rollHeaderShowsDirectionAndDistanceRolls() {
		stubDirection(Direction.EAST);
		ReportKickoffScatter report = new ReportKickoffScatter(new FieldCoordinate(0, 0), Direction.EAST, 2, 4);
		List<Run> runs = render(new KickoffScatterMessage(), report);

		assertTrue(runs.stream().anyMatch(r -> "Kick-off Scatter Roll [ 2 ][ 4 ]".equals(r.text)));
	}

	@Test
	public void indentIsLeftAt1AfterRender() {
		stubDirection(Direction.EAST);
		ReportKickoffScatter report = new ReportKickoffScatter(new FieldCoordinate(0, 0), Direction.EAST, 2, 4);
		render(new KickoffScatterMessage(), report);

		assertEquals(1, statusReport.getIndent());
	}
}
