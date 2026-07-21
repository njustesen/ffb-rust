package com.fumbbl.ffb.report.mixed;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.report.ReportTestUtil;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

public class ReportNervesOfSteelTest {

	private ReportNervesOfSteel make() {
		return new ReportNervesOfSteel("p1", "PASS");
	}

	@Test
	public void serializationRoundTrip() {
		ReportNervesOfSteel original = make();
		JsonObject json = (JsonObject) original.toJsonValue();
		ReportNervesOfSteel restored = (ReportNervesOfSteel) new ReportNervesOfSteel().initFrom(ReportTestUtil.source(), json);
		assertEquals(original.getPlayerId(), restored.getPlayerId());
		assertEquals(original.getBallAction(), restored.getBallAction());
		assertEquals(original.isBomb(), restored.isBomb());
	}

	@Test
	public void toJsonValueHasReportId() {
		JsonObject json = (JsonObject) make().toJsonValue();
		assertEquals("nervesOfSteel", json.get("reportId").asString());
	}
}
