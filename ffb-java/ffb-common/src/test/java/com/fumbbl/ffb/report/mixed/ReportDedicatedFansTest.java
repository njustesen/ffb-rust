package com.fumbbl.ffb.report.mixed;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.report.ReportTestUtil;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

public class ReportDedicatedFansTest {

	private ReportDedicatedFans make() {
		return new ReportDedicatedFans(3, 1, 2, 0, "away", true);
	}

	@Test
	public void serializationRoundTrip() {
		ReportDedicatedFans original = make();
		JsonObject json = (JsonObject) original.toJsonValue();
		ReportDedicatedFans restored = new ReportDedicatedFans().initFrom(ReportTestUtil.source(), json);
		assertEquals(original.getRollHome(), restored.getRollHome());
		assertEquals(original.getModifierHome(), restored.getModifierHome());
		assertEquals(original.getRollAway(), restored.getRollAway());
		assertEquals(original.getModifierAway(), restored.getModifierAway());
		assertEquals(original.getConcededTeam(), restored.getConcededTeam());
		assertEquals(original.isConceded(), restored.isConceded());
	}

	@Test
	public void toJsonValueHasReportId() {
		JsonObject json = (JsonObject) make().toJsonValue();
		assertEquals("dedicatedFans", json.get("reportId").asString());
	}
}
