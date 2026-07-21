package com.fumbbl.ffb.report.bb2016;

import com.eclipsesource.json.JsonValue;
import com.fumbbl.ffb.report.ReportTestUtil;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

public class ReportNervesOfSteelTest {

	private ReportNervesOfSteel make() {
		return new ReportNervesOfSteel("p1", "pass");
	}

	@Test
	public void serializationRoundTrip() {
		ReportNervesOfSteel original = make();
		JsonValue json = original.toJsonValue();
		ReportNervesOfSteel restored = (ReportNervesOfSteel) new ReportNervesOfSteel().initFrom(ReportTestUtil.source(), json);
		assertEquals(original.getPlayerId(), restored.getPlayerId());
		assertEquals(original.getBallAction(), restored.getBallAction());
	}

	@Test
	public void toJsonValueHasReportId() {
		JsonValue json = make().toJsonValue();
		assertEquals("nervesOfSteel", json.asObject().get("reportId").asString());
	}
}
