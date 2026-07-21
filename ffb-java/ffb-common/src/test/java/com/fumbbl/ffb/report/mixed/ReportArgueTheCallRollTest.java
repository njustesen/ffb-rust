package com.fumbbl.ffb.report.mixed;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.report.ReportTestUtil;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

public class ReportArgueTheCallRollTest {

	private ReportArgueTheCallRoll make() {
		return new ReportArgueTheCallRoll("p1", true, false, 5, true, false, 1);
	}

	@Test
	public void serializationRoundTrip() {
		ReportArgueTheCallRoll original = make();
		JsonObject json = (JsonObject) original.toJsonValue();
		ReportArgueTheCallRoll restored = new ReportArgueTheCallRoll().initFrom(ReportTestUtil.source(), json);
		assertEquals(original.getPlayerId(), restored.getPlayerId());
		assertEquals(original.isSuccessful(), restored.isSuccessful());
		assertEquals(original.isCoachBanned(), restored.isCoachBanned());
		assertEquals(original.getRoll(), restored.getRoll());
		assertEquals(original.isStaysOnPitch(), restored.isStaysOnPitch());
		assertEquals(original.isFriendsWithRef(), restored.isFriendsWithRef());
		assertEquals(original.getBiasedRefs(), restored.getBiasedRefs());
	}

	@Test
	public void toJsonValueHasReportId() {
		JsonObject json = (JsonObject) make().toJsonValue();
		assertEquals("argueTheCall", json.get("reportId").asString());
	}
}
