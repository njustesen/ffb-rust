package com.fumbbl.ffb.report.mixed;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.report.ReportTestUtil;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

public class ReportCloudBursterTest {

	private ReportCloudBurster make() {
		return new ReportCloudBurster("t1", "i1", "team1");
	}

	@Test
	public void serializationRoundTrip() {
		ReportCloudBurster original = make();
		JsonObject json = (JsonObject) original.toJsonValue();
		ReportCloudBurster restored = new ReportCloudBurster().initFrom(ReportTestUtil.source(), json);
		assertEquals(original.getThrowerId(), restored.getThrowerId());
		assertEquals(original.getInterceptorId(), restored.getInterceptorId());
	}

	@Test
	public void toJsonValueHasReportId() {
		JsonObject json = (JsonObject) make().toJsonValue();
		assertEquals("cloudBurster", json.get("reportId").asString());
	}
}
