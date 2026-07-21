package com.fumbbl.ffb.report.mixed;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.report.ReportTestUtil;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

public class ReportIndomitableTest {

	private ReportIndomitable make() {
		return new ReportIndomitable("p1", "d1");
	}

	@Test
	public void serializationRoundTrip() {
		ReportIndomitable original = make();
		JsonObject json = (JsonObject) original.toJsonValue();
		ReportIndomitable restored = new ReportIndomitable().initFrom(ReportTestUtil.source(), json);
		assertEquals(original.getPlayerId(), restored.getPlayerId());
		assertEquals(original.getDefenderId(), restored.getDefenderId());
	}

	@Test
	public void toJsonValueHasReportId() {
		JsonObject json = (JsonObject) make().toJsonValue();
		assertEquals("indomitable", json.get("reportId").asString());
	}
}
