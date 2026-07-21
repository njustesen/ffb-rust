package com.fumbbl.ffb.client.report.bb2016;

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

	private void stubMapToLocal() {
		given(client.getUserInterface().getPitchDimensionProvider().mapToLocal(any(Direction.class)))
			.willAnswer(invocation -> invocation.getArgument(0));
	}

	@Test
	public void getKeyIsScatterPlayer() {
		assertEquals("scatterPlayer", new ScatterPlayerMessage().getKey());
	}

	@Test
	public void reportsScatterAndMovement() {
		stubMapToLocal();

		ReportScatterPlayer report = new ReportScatterPlayer(
			new FieldCoordinate(3, 4), new FieldCoordinate(5, 6), new Direction[]{Direction.NORTH}, new int[]{2}, true);
		List<Run> runs = render(new ScatterPlayerMessage(), report);

		assertEquals("Scatter Roll [ 2 ] North", runs.get(0).text);
		assertEquals("Player scatters from square (3,4) to square (5,6).", runs.get(2).text);
	}

	@Test
	public void emptyRollsProduceNoOutput() {
		ReportScatterPlayer report = new ReportScatterPlayer(
			new FieldCoordinate(0, 0), new FieldCoordinate(0, 0), new Direction[]{}, new int[]{}, null);
		List<Run> runs = render(new ScatterPlayerMessage(), report);

		assertTrue(runs.isEmpty());
	}

	@Test
	public void multipleRollsUsePluralLabel() {
		stubMapToLocal();

		ReportScatterPlayer report = new ReportScatterPlayer(
			new FieldCoordinate(1, 1), new FieldCoordinate(2, 2), new Direction[]{Direction.EAST, Direction.SOUTH}, new int[]{3, 5}, false);
		List<Run> runs = render(new ScatterPlayerMessage(), report);

		assertEquals("Scatter Rolls [ 3, 5 ] East, South", runs.get(0).text);
	}
}
