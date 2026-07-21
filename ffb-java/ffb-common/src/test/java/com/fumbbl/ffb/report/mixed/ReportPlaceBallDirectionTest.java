package com.fumbbl.ffb.report.mixed;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.Direction;
import com.fumbbl.ffb.report.ReportTestUtil;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

public class ReportPlaceBallDirectionTest {

	private ReportPlaceBallDirection make() {
		return new ReportPlaceBallDirection("p1", Direction.NORTH);
	}

	@Test
	public void serializationRoundTrip() {
		ReportPlaceBallDirection original = make();
		JsonObject json = original.toJsonValue().asObject();
		ReportPlaceBallDirection restored = new ReportPlaceBallDirection().initFrom(ReportTestUtil.source(), json);
		assertEquals(original.getPlayerId(), restored.getPlayerId());
		assertEquals(original.getDirection(), restored.getDirection());
	}

	@Test
	public void toJsonValueHasReportId() {
		JsonObject json = make().toJsonValue().asObject();
		assertEquals("placedBallDirection", json.get("reportId").asString());
	}
}
