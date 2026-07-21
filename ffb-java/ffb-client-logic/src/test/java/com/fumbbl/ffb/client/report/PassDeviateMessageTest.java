package com.fumbbl.ffb.client.report;

import com.fumbbl.ffb.Direction;
import com.fumbbl.ffb.FieldCoordinate;
import com.fumbbl.ffb.report.ReportPassDeviate;

import org.junit.jupiter.api.Test;

import java.util.List;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.BDDMockito.given;

class PassDeviateMessageTest extends ReportMessageTestBase {

	@Test
	public void passDeviateUsesBallWording() {
		given(client.getUserInterface().getPitchDimensionProvider().mapToLocal(Direction.NORTH))
			.willReturn(Direction.NORTH);

		ReportPassDeviate report = new ReportPassDeviate(new FieldCoordinate(0, 0), Direction.NORTH, 3, 2, false);
		List<Run> runs = render(new PassDeviateMessage(), report);

		assertTrue(runs.stream().anyMatch(r -> "Pass Deviates [ 3 ][ 2 ]".equals(r.text)));
		assertTrue(runs.stream().anyMatch(r -> "The ball will land 2 squares north from the thrower.".equals(r.text)));
	}

	@Test
	public void ttmDeviateUsesPlayerWording() {
		given(client.getUserInterface().getPitchDimensionProvider().mapToLocal(Direction.SOUTH))
			.willReturn(Direction.SOUTH);

		ReportPassDeviate report = new ReportPassDeviate(new FieldCoordinate(0, 0), Direction.SOUTH, 1, 1, true);
		List<Run> runs = render(new PassDeviateMessage(), report);

		assertTrue(runs.stream().anyMatch(r -> "Throw Team Mate Deviates [ 1 ][ 1 ]".equals(r.text)));
		assertTrue(runs.stream().anyMatch(r -> "The player will land 1 square south from the thrower.".equals(r.text)));
	}

	@Test
	public void indentIsLeftAt1AfterRender() {
		given(client.getUserInterface().getPitchDimensionProvider().mapToLocal(Direction.EAST))
			.willReturn(Direction.EAST);

		ReportPassDeviate report = new ReportPassDeviate(new FieldCoordinate(0, 0), Direction.EAST, 2, 4, false);
		render(new PassDeviateMessage(), report);

		assertEquals(1, statusReport.getIndent());
	}
}
