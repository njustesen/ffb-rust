package com.fumbbl.ffb.report.bb2025;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.Direction;
import com.fumbbl.ffb.report.ReportTestUtil;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

public class ReportPuntDirectionTest {
	private ReportPuntDirection make() {
		return new ReportPuntDirection(Direction.NORTH, 3, "p1", false);
	}

	@Test
	public void serializationRoundTrip() {
		ReportPuntDirection original = make();
		JsonObject json = original.toJsonValue();
		ReportPuntDirection restored = new ReportPuntDirection().initFrom(ReportTestUtil.source(), json);
		assertEquals(original.getDirection(), restored.getDirection());
		assertEquals(original.getDirectionRoll(), restored.getDirectionRoll());
		assertEquals(original.getPlayerId(), restored.getPlayerId());
		assertEquals(original.isOutOfBounds(), restored.isOutOfBounds());
	}

	@Test
	public void toJsonValueHasReportId() {
		JsonObject json = make().toJsonValue();
		assertEquals("puntDirectionRoll", json.get("reportId").asString());
	}
}
