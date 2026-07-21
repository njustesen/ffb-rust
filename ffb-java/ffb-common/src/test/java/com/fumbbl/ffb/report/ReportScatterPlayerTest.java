package com.fumbbl.ffb.report;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.Direction;
import com.fumbbl.ffb.FieldCoordinate;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertArrayEquals;
import static org.junit.jupiter.api.Assertions.assertEquals;

public class ReportScatterPlayerTest {

	private ReportScatterPlayer make() {
		return new ReportScatterPlayer(new FieldCoordinate(3, 5), new FieldCoordinate(4, 5),
				new Direction[]{Direction.EAST}, new int[]{3}, Boolean.TRUE);
	}

	@Test
	public void serializationRoundTrip() {
		ReportScatterPlayer original = make();
		JsonObject json = original.toJsonValue();
		ReportScatterPlayer restored = new ReportScatterPlayer().initFrom(ReportTestUtil.source(), json);
		assertEquals(original.getStartCoordinate(), restored.getStartCoordinate());
		assertEquals(original.getEndCoordinate(), restored.getEndCoordinate());
		assertArrayEquals(original.getDirections(), restored.getDirections());
		assertArrayEquals(original.getRolls(), restored.getRolls());
		assertEquals(original.getScatter(), restored.getScatter());
	}

	@Test
	public void toJsonValueHasReportId() {
		JsonObject json = make().toJsonValue();
		assertEquals("scatterPlayer", json.get("reportId").asString());
	}
}
