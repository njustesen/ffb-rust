package com.fumbbl.ffb.report.mixed;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.report.ReportTestUtil;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

public class ReportFumblerooskieTest {

	private ReportFumblerooskie make() {
		return new ReportFumblerooskie("p1", true);
	}

	@Test
	public void serializationRoundTrip() {
		ReportFumblerooskie original = make();
		JsonObject json = (JsonObject) original.toJsonValue();
		ReportFumblerooskie restored = new ReportFumblerooskie().initFrom(ReportTestUtil.source(), json);
		assertEquals(original.getPlayerId(), restored.getPlayerId());
		assertEquals(original.isUsed(), restored.isUsed());
	}

	@Test
	public void toJsonValueHasReportId() {
		JsonObject json = (JsonObject) make().toJsonValue();
		assertEquals("fumblerooskie", json.get("reportId").asString());
	}
}
