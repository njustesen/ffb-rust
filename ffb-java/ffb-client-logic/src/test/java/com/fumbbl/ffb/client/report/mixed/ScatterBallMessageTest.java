package com.fumbbl.ffb.client.report.mixed;

import com.fumbbl.ffb.Direction;
import com.fumbbl.ffb.client.TextStyle;
import com.fumbbl.ffb.client.report.ReportMessageTestBase;
import com.fumbbl.ffb.report.ReportScatterBall;
import org.junit.jupiter.api.Test;

import java.util.List;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.ArgumentMatchers.any;
import static org.mockito.BDDMockito.given;

class ScatterBallMessageTest extends ReportMessageTestBase {

	private void stubDirections() {
		given(client.getUserInterface().getPitchDimensionProvider().mapToLocal(any(Direction.class)))
			.willAnswer(invocation -> invocation.getArgument(0));
	}

	@Test
	public void singleRollUsesBounceWording() {
		stubDirections();
		ReportScatterBall report = new ReportScatterBall(new Direction[] { Direction.NORTH }, new int[] { 3 }, false);
		List<Run> runs = render(new ScatterBallMessage(), report);

		assertEquals("Bounce Roll [ 3 ] North", runs.get(0).text);
		assertEquals(TextStyle.ROLL, runs.get(0).textStyle);
	}

	@Test
	public void multiRollUsesScatterWordingAndJoinsDirections() {
		stubDirections();
		ReportScatterBall report = new ReportScatterBall(new Direction[] { Direction.NORTH, Direction.EAST }, new int[] { 3, 5 }, false);
		List<Run> runs = render(new ScatterBallMessage(), report);

		assertEquals("Scatter Rolls [ 3, 5 ] North, East", runs.get(0).text);
	}

	@Test
	public void gustOfWindPrintsExtraLineAtIncrementedIndentAndRestores() {
		stubDirections();
		ReportScatterBall report = new ReportScatterBall(new Direction[] { Direction.NORTH }, new int[] { 3 }, true);
		ScatterBallMessage handler = new ScatterBallMessage();
		List<Run> runs = render(handler, report);

		assertEquals("A gust of wind scatters the ball.", runs.get(0).text);
		// indent restored to 0 after rendering.
		assertEquals(0, handler.getIndent());
	}

	@Test
	public void emptyRollsPrintsNothingButGustLine() {
		ReportScatterBall report = new ReportScatterBall(new Direction[0], new int[0], true);
		List<Run> runs = render(new ScatterBallMessage(), report);

		assertEquals(1, runs.stream().filter(r -> r.text != null).count());
		assertTrue(runs.stream().anyMatch(r -> "A gust of wind scatters the ball.".equals(r.text)));
	}
}
