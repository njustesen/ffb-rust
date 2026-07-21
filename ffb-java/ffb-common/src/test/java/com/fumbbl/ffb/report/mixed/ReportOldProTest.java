package com.fumbbl.ffb.report.mixed;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.report.ReportTestUtil;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

public class ReportOldProTest {

	private ReportOldPro make() {
		return new ReportOldPro("p1", 3, 2, false);
	}

	@Test
	public void serializationRoundTrip() {
		ReportOldPro original = make();
		JsonObject json = original.toJsonValue().asObject();
		ReportOldPro restored = new ReportOldPro().initFrom(ReportTestUtil.source(), json);
		assertEquals(original.getPlayerId(), restored.getPlayerId());
		assertEquals(original.getOldValue(), restored.getOldValue());
		assertEquals(original.getNewValue(), restored.getNewValue());
		assertEquals(original.isSelfInflicted(), restored.isSelfInflicted());
	}

	@Test
	public void toJsonValueHasReportId() {
		JsonObject json = make().toJsonValue().asObject();
		assertEquals("oldPro", json.get("reportId").asString());
	}
}
