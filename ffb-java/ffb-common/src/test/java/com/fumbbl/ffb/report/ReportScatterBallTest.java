package com.fumbbl.ffb.report;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.Direction;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertArrayEquals;
import static org.junit.jupiter.api.Assertions.assertEquals;

public class ReportScatterBallTest {

	private ReportScatterBall make() {
		return new ReportScatterBall(new Direction[]{Direction.NORTH, Direction.EAST}, new int[]{3, 5}, false);
	}

	@Test
	public void serializationRoundTrip() {
		ReportScatterBall original = make();
		JsonObject json = original.toJsonValue();
		ReportScatterBall restored = new ReportScatterBall().initFrom(ReportTestUtil.source(), json);
		assertArrayEquals(original.getDirections(), restored.getDirections());
		assertArrayEquals(original.getRolls(), restored.getRolls());
		assertEquals(original.isGustOfWind(), restored.isGustOfWind());
	}

	@Test
	public void toJsonValueHasReportId() {
		JsonObject json = make().toJsonValue();
		assertEquals("scatterBall", json.get("reportId").asString());
	}
}
