package com.fumbbl.ffb.report.mixed;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.report.ReportTestUtil;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

public class ReportSelectGazeTargetTest {

	private ReportSelectGazeTarget make() {
		return new ReportSelectGazeTarget("a1", "d1");
	}

	@Test
	public void serializationRoundTrip() {
		ReportSelectGazeTarget original = make();
		JsonObject json = original.toJsonValue().asObject();
		ReportSelectGazeTarget restored = new ReportSelectGazeTarget().initFrom(ReportTestUtil.source(), json);
		assertEquals(original.getAttacker(), restored.getAttacker());
		assertEquals(original.getDefender(), restored.getDefender());
	}

	@Test
	public void toJsonValueHasReportId() {
		JsonObject json = make().toJsonValue().asObject();
		assertEquals("selectGazeTarget", json.get("reportId").asString());
	}
}
