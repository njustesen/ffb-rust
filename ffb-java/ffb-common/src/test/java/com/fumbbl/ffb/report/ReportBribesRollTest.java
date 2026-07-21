package com.fumbbl.ffb.report;

import com.eclipsesource.json.JsonObject;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

public class ReportBribesRollTest {

	private ReportBribesRoll make() {
		return new ReportBribesRoll("p1", true, 4);
	}

	@Test
	public void serializationRoundTrip() {
		ReportBribesRoll original = make();
		JsonObject json = original.toJsonValue();
		ReportBribesRoll restored = new ReportBribesRoll().initFrom(ReportTestUtil.source(), json);
		assertEquals(original.getPlayerId(), restored.getPlayerId());
		assertEquals(original.isSuccessful(), restored.isSuccessful());
		assertEquals(original.getRoll(), restored.getRoll());
	}

	@Test
	public void toJsonValueHasReportId() {
		JsonObject json = make().toJsonValue();
		assertEquals("bribesRoll", json.get("reportId").asString());
	}
}
