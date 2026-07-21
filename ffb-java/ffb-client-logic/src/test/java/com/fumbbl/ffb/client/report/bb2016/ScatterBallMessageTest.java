package com.fumbbl.ffb.client.report.bb2016;

import com.fumbbl.ffb.Direction;
import com.fumbbl.ffb.client.report.ReportMessageTestBase;
import com.fumbbl.ffb.report.ReportScatterBall;
import org.junit.jupiter.api.Test;

import java.util.List;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.ArgumentMatchers.any;
import static org.mockito.BDDMockito.given;

class ScatterBallMessageTest extends ReportMessageTestBase {

	private void stubMapToLocal() {
		given(client.getUserInterface().getPitchDimensionProvider().mapToLocal(any(Direction.class)))
			.willAnswer(invocation -> invocation.getArgument(0));
	}

	@Test
	public void getKeyIsScatterBall() {
		assertEquals("scatterBall", new ScatterBallMessage().getKey());
	}

	@Test
	public void reportsSingleScatterRoll() {
		stubMapToLocal();

		ReportScatterBall report = new ReportScatterBall(new Direction[]{Direction.NORTH}, new int[]{3}, false);
		List<Run> runs = render(new ScatterBallMessage(), report);

		assertEquals("Scatter Roll [ 3 ] North", runs.get(0).text);
	}

	@Test
	public void gustOfWindBumpsIndentAndReportsMessage() {
		stubMapToLocal();

		ReportScatterBall report = new ReportScatterBall(new Direction[]{Direction.EAST, Direction.SOUTH}, new int[]{2, 4}, true);
		List<Run> runs = render(new ScatterBallMessage(), report);

		assertEquals("A gust of wind scatters the ball.", runs.get(0).text);
		assertEquals(0, statusReport.getIndent());
	}

	@Test
	public void emptyRollsProduceNoRollLine() {
		ReportScatterBall report = new ReportScatterBall(new Direction[]{}, new int[]{}, false);
		List<Run> runs = render(new ScatterBallMessage(), report);

		assertTrue(runs.isEmpty());
	}
}
