package com.fumbbl.ffb.report.mixed;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.report.ReportTestUtil;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

public class ReportFanFactorTest {

	private ReportFanFactor make() {
		return new ReportFanFactor("team1", 3, 2);
	}

	@Test
	public void serializationRoundTrip() {
		ReportFanFactor original = make();
		JsonObject json = (JsonObject) original.toJsonValue();
		ReportFanFactor restored = new ReportFanFactor().initFrom(ReportTestUtil.source(), json);
		assertEquals(original.getRoll(), restored.getRoll());
		assertEquals(original.getDedicatedFans(), restored.getDedicatedFans());
		assertEquals(original.getResult(), restored.getResult());
		assertEquals(original.getTeamId(), restored.getTeamId());
	}

	@Test
	public void toJsonValueHasReportId() {
		JsonObject json = (JsonObject) make().toJsonValue();
		assertEquals("fanFactor", json.get("reportId").asString());
	}
}
