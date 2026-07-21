package com.fumbbl.ffb.client.report.mixed;

import com.fumbbl.ffb.Direction;
import com.fumbbl.ffb.FieldCoordinate;
import com.fumbbl.ffb.client.report.ReportMessageTestBase;
import com.fumbbl.ffb.report.ReportScatterPlayer;
import org.junit.jupiter.api.Test;

import java.util.List;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.ArgumentMatchers.any;
import static org.mockito.BDDMockito.given;

class ScatterPlayerMessageTest extends ReportMessageTestBase {

	private void stubDirections() {
		given(client.getUserInterface().getPitchDimensionProvider().mapToLocal(any(Direction.class)))
			.willAnswer(invocation -> invocation.getArgument(0));
	}

	@Test
	public void singleRollWithoutExplicitScatterFlagBounces() {
		stubDirections();
		ReportScatterPlayer report = new ReportScatterPlayer(
			new FieldCoordinate(3, 5), new FieldCoordinate(4, 5), new Direction[] { Direction.EAST }, new int[] { 3 });
		List<Run> runs = render(new ScatterPlayerMessage(), report);

		assertEquals("Bounce Roll [ 3 ] East", runs.get(0).text);
		String second = runs.stream().filter(r -> r.text != null).map(r -> r.text).skip(1).findFirst().orElseThrow();
		assertEquals("Player bounces from square (3,5) to square (4,5).", second);
	}

	@Test
	public void multiRollWithoutExplicitScatterFlagScatters() {
		stubDirections();
		ReportScatterPlayer report = new ReportScatterPlayer(
			new FieldCoordinate(0, 0), new FieldCoordinate(2, 1), new Direction[] { Direction.NORTH, Direction.EAST }, new int[] { 3, 4 });
		List<Run> runs = render(new ScatterPlayerMessage(), report);

		assertEquals("Scatter Rolls [ 3, 4 ] North, East", runs.get(0).text);
		String second = runs.stream().filter(r -> r.text != null).map(r -> r.text).skip(1).findFirst().orElseThrow();
		assertEquals("Player scatters from square (0,0) to square (2,1).", second);
	}

	@Test
	public void explicitScatterFlagOverridesRollCount() {
		stubDirections();
		ReportScatterPlayer report = new ReportScatterPlayer(
			new FieldCoordinate(0, 0), new FieldCoordinate(1, 0), new Direction[] { Direction.WEST }, new int[] { 4 }, true);
		List<Run> runs = render(new ScatterPlayerMessage(), report);

		assertEquals("Scatter Rolls [ 4 ] West", runs.get(0).text);
	}

	@Test
	public void emptyRollsRendersNothing() {
		ReportScatterPlayer report = new ReportScatterPlayer(
			new FieldCoordinate(0, 0), new FieldCoordinate(1, 0), new Direction[0], new int[0]);
		List<Run> runs = render(new ScatterPlayerMessage(), report);

		assertTrue(runs.isEmpty());
	}
}
