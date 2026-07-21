package com.fumbbl.ffb.client.report;

import com.fumbbl.ffb.Direction;
import com.fumbbl.ffb.report.ReportThrowIn;
import org.junit.jupiter.api.Test;

import java.util.List;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.BDDMockito.given;

class ThrowInMessageTest extends ReportMessageTestBase {

	private void givenIdentityMapping(Direction direction) {
		given(client.getUserInterface().getPitchDimensionProvider().mapToLocal(direction)).willReturn(direction);
	}

	@Test
	public void fullThrowInRendersAllLines() {
		givenIdentityMapping(Direction.NORTH);

		ReportThrowIn report = new ReportThrowIn(Direction.NORTH, 3, new int[]{2, 4});
		List<Run> runs = render(new ThrowInMessage(), report);

		assertTrue(runs.stream().anyMatch(r -> "Throw In Direction Roll [ 3 ] North".equals(r.text)));
		assertTrue(runs.stream().anyMatch(r -> "Throw In Distance Roll [ 2 ][ 4 ]".equals(r.text)));
		assertTrue(runs.stream().anyMatch(r -> "The fans throw the ball back onto the pitch.".equals(r.text)));
		assertTrue(runs.stream().anyMatch(r -> "It lands 6 squares North".equals(r.text)));
	}

	@Test
	public void singleElementDistanceRollRendersNothing() {
		ReportThrowIn report = new ReportThrowIn(Direction.SOUTH, 1, new int[]{2});
		List<Run> runs = render(new ThrowInMessage(), report);

		assertTrue(runs.isEmpty());
	}

	@Test
	public void emptyDistanceRollRendersNothing() {
		ReportThrowIn report = new ReportThrowIn(Direction.EAST, 1, new int[]{});
		List<Run> runs = render(new ThrowInMessage(), report);

		assertTrue(runs.isEmpty());
	}

	@Test
	public void reportIdIsThrowIn() {
		assertEquals("throwIn", new ThrowInMessage().getKey());
	}
}
