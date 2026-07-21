package com.fumbbl.ffb.report.mixed;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.report.ReportTestUtil;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

public class ReportStallerDetectedTest {

	private ReportStallerDetected make() {
		return new ReportStallerDetected("p1");
	}

	@Test
	public void serializationRoundTrip() {
		ReportStallerDetected original = make();
		JsonObject json = original.toJsonValue().asObject();
		ReportStallerDetected restored = new ReportStallerDetected().initFrom(ReportTestUtil.source(), json);
		assertEquals(original.getPlayerId(), restored.getPlayerId());
	}

	@Test
	public void toJsonValueHasReportId() {
		JsonObject json = make().toJsonValue().asObject();
		assertEquals("stallerDetected", json.get("reportId").asString());
	}
}
