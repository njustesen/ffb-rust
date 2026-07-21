package com.fumbbl.ffb.report.mixed;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.report.ReportTestUtil;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

public class ReportDoubleHiredStaffTest {

	private ReportDoubleHiredStaff make() {
		return new ReportDoubleHiredStaff("apothecary");
	}

	@Test
	public void serializationRoundTrip() {
		ReportDoubleHiredStaff original = make();
		JsonObject json = (JsonObject) original.toJsonValue();
		ReportDoubleHiredStaff restored = new ReportDoubleHiredStaff().initFrom(ReportTestUtil.source(), json);
		assertEquals(original.getStaffName(), restored.getStaffName());
	}

	@Test
	public void toJsonValueHasReportId() {
		JsonObject json = (JsonObject) make().toJsonValue();
		assertEquals("doubleHiredStaff", json.get("reportId").asString());
	}
}
