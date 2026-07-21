package com.fumbbl.ffb.report.mixed;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.report.ReportTestUtil;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

public class ReportBiasedRefTest {

	private ReportBiasedRef make() {
		return new ReportBiasedRef(3, true);
	}

	@Test
	public void serializationRoundTrip() {
		ReportBiasedRef original = make();
		JsonObject json = (JsonObject) original.toJsonValue();
		ReportBiasedRef restored = new ReportBiasedRef().initFrom(ReportTestUtil.source(), json);
		assertEquals(original.isFoulSpotted(), restored.isFoulSpotted());
		assertEquals(original.getRoll(), restored.getRoll());
	}

	@Test
	public void toJsonValueHasReportId() {
		JsonObject json = (JsonObject) make().toJsonValue();
		assertEquals("biasedRef", json.get("reportId").asString());
	}
}
