package com.fumbbl.ffb.report;

import com.eclipsesource.json.JsonObject;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

public class ReportRaiseDeadTest {

	private ReportRaiseDead make() {
		return new ReportRaiseDead("p1", "Zombie", true);
	}

	@Test
	public void serializationRoundTrip() {
		ReportRaiseDead original = make();
		JsonObject json = original.toJsonValue();
		ReportRaiseDead restored = new ReportRaiseDead().initFrom(ReportTestUtil.source(), json);
		assertEquals(original.getPlayerId(), restored.getPlayerId());
		assertEquals(original.getPosition(), restored.getPosition());
		assertEquals(original.isNurglesRot(), restored.isNurglesRot());
	}

	@Test
	public void toJsonValueHasReportId() {
		JsonObject json = make().toJsonValue();
		assertEquals("raiseDead", json.get("reportId").asString());
	}
}
