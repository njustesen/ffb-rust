package com.fumbbl.ffb.report.bb2020;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.report.ReportTestUtil;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

/**
 * Mirror of ffb-rust crates/ffb-model/src/report/bb2020/report_two_for_one.rs tests.
 */
public class ReportTwoForOneTest {

	private ReportTwoForOne make() {
		return new ReportTwoForOne("p1", "p2", true);
	}

	@Test
	public void serializationRoundTrip() {
		ReportTwoForOne original = make();
		JsonObject json = original.toJsonValue();
		ReportTwoForOne restored = new ReportTwoForOne().initFrom(ReportTestUtil.source(), json);
		assertEquals(original.getPlayerId(), restored.getPlayerId());
		assertEquals(original.getPartnerId(), restored.getPartnerId());
		assertEquals(original.isUsed(), restored.isUsed());
	}

	@Test
	public void toJsonValueHasReportId() {
		JsonObject json = make().toJsonValue();
		assertEquals("twoForOne", json.get("reportId").asString());
	}
}
