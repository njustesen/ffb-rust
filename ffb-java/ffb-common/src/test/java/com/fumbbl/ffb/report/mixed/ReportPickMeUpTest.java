package com.fumbbl.ffb.report.mixed;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.report.ReportTestUtil;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

public class ReportPickMeUpTest {

	private ReportPickMeUp make() {
		return new ReportPickMeUp("p1", 5, true);
	}

	@Test
	public void serializationRoundTrip() {
		ReportPickMeUp original = make();
		JsonObject json = original.toJsonValue().asObject();
		ReportPickMeUp restored = new ReportPickMeUp().initFrom(ReportTestUtil.source(), json);
		assertEquals(original.getPlayerId(), restored.getPlayerId());
		assertEquals(original.isSuccess(), restored.isSuccess());
		assertEquals(original.getRoll(), restored.getRoll());
	}

	@Test
	public void toJsonValueHasReportId() {
		JsonObject json = make().toJsonValue().asObject();
		assertEquals("pickMeUp", json.get("reportId").asString());
	}
}
