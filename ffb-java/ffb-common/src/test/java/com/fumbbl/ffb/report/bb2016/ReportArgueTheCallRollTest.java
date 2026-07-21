package com.fumbbl.ffb.report.bb2016;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.report.ReportTestUtil;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

public class ReportArgueTheCallRollTest {

	private ReportArgueTheCallRoll make() {
		return new ReportArgueTheCallRoll("p1", true, false, 5);
	}

	@Test
	public void serializationRoundTrip() {
		ReportArgueTheCallRoll original = make();
		JsonObject json = original.toJsonValue();
		ReportArgueTheCallRoll restored = new ReportArgueTheCallRoll().initFrom(ReportTestUtil.source(), json);
		assertEquals(original.getPlayerId(), restored.getPlayerId());
		assertEquals(original.isSuccessful(), restored.isSuccessful());
		assertEquals(original.isCoachBanned(), restored.isCoachBanned());
		assertEquals(original.getRoll(), restored.getRoll());
	}

	@Test
	public void toJsonValueHasReportId() {
		JsonObject json = make().toJsonValue();
		assertEquals("argueTheCall", json.get("reportId").asString());
	}
}
