package com.fumbbl.ffb.client.report.bb2025;

import com.fumbbl.ffb.client.report.ReportMessageTestBase;
import com.fumbbl.ffb.report.ReportId;
import com.fumbbl.ffb.report.bb2025.ReportPuntDistance;
import org.junit.jupiter.api.Test;

import java.util.List;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;

class PuntDistanceMessageTest extends ReportMessageTestBase {

	@Test
	public void positiveRollReportsDistance() {
		ReportPuntDistance report = new ReportPuntDistance(4, false);
		List<Run> runs = render(new PuntDistanceMessage(), report);

		assertTrue(runs.stream().anyMatch(r -> "Punt Distance Roll [ 4 ]".equals(r.text)));
		assertTrue(runs.stream().anyMatch(r -> "The ball is punted 4 squares".equals(r.text)));
	}

	@Test
	public void outOfBoundsAppendsText() {
		ReportPuntDistance report = new ReportPuntDistance(3, true);
		List<Run> runs = render(new PuntDistanceMessage(), report);

		assertTrue(runs.stream().anyMatch(r -> "The ball is punted 3 squares putting it out of bounds".equals(r.text)));
	}

	@Test
	public void zeroRollPrintsNothing() {
		ReportPuntDistance report = new ReportPuntDistance(0, false);
		List<Run> runs = render(new PuntDistanceMessage(), report);

		assertTrue(runs.isEmpty());
	}

	@Test
	public void negativeRollPrintsNothing() {
		ReportPuntDistance report = new ReportPuntDistance(-1, false);
		List<Run> runs = render(new PuntDistanceMessage(), report);

		assertTrue(runs.isEmpty());
	}

	@Test
	public void reportIdIsPuntDistanceRoll() {
		assertEquals(ReportId.PUNT_DISTANCE_ROLL.getKey(), new PuntDistanceMessage().getKey());
	}
}
