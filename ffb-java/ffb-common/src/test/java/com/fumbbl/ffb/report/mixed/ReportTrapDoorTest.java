package com.fumbbl.ffb.report.mixed;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.report.ReportTestUtil;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

public class ReportTrapDoorTest {

	private ReportTrapDoor make() {
		return new ReportTrapDoor("p1", 4, true);
	}

	@Test
	public void serializationRoundTrip() {
		ReportTrapDoor original = make();
		JsonObject json = original.toJsonValue().asObject();
		ReportTrapDoor restored = new ReportTrapDoor().initFrom(ReportTestUtil.source(), json);
		assertEquals(original.getPlayerId(), restored.getPlayerId());
		assertEquals(original.isEscaped(), restored.isEscaped());
		assertEquals(original.getRoll(), restored.getRoll());
	}

	@Test
	public void toJsonValueHasReportId() {
		JsonObject json = make().toJsonValue().asObject();
		assertEquals("trapDoor", json.get("reportId").asString());
	}
}
