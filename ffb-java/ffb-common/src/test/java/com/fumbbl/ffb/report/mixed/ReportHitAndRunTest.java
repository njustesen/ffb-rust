package com.fumbbl.ffb.report.mixed;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.Direction;
import com.fumbbl.ffb.report.ReportTestUtil;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

public class ReportHitAndRunTest {

	private ReportHitAndRun make() {
		return new ReportHitAndRun("p1", Direction.NORTH);
	}

	@Test
	public void serializationRoundTrip() {
		ReportHitAndRun original = make();
		JsonObject json = (JsonObject) original.toJsonValue();
		ReportHitAndRun restored = new ReportHitAndRun().initFrom(ReportTestUtil.source(), json);
		assertEquals(original.getPlayerId(), restored.getPlayerId());
		assertEquals(original.getDirection(), restored.getDirection());
	}

	@Test
	public void toJsonValueHasReportId() {
		JsonObject json = (JsonObject) make().toJsonValue();
		assertEquals("hitAndRun", json.get("reportId").asString());
	}
}
