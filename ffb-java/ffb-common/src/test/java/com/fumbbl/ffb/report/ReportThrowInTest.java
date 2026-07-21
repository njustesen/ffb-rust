package com.fumbbl.ffb.report;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.Direction;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertArrayEquals;
import static org.junit.jupiter.api.Assertions.assertEquals;

public class ReportThrowInTest {

	private ReportThrowIn make() {
		return new ReportThrowIn(Direction.NORTH, 3, new int[]{2, 4});
	}

	@Test
	public void serializationRoundTrip() {
		ReportThrowIn original = make();
		JsonObject json = original.toJsonValue();
		ReportThrowIn restored = new ReportThrowIn().initFrom(ReportTestUtil.source(), json);
		assertEquals(original.getDirection(), restored.getDirection());
		assertEquals(original.getDirectionRoll(), restored.getDirectionRoll());
		assertArrayEquals(original.getDistanceRoll(), restored.getDistanceRoll());
	}

	@Test
	public void toJsonValueHasReportId() {
		JsonObject json = make().toJsonValue();
		assertEquals("throwIn", json.get("reportId").asString());
	}
}
