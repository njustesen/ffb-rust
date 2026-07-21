package com.fumbbl.ffb.report.mixed;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.report.ReportTestUtil;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

public class ReportBalefulHexRollTest {

	private ReportBalefulHexRoll make() {
		return new ReportBalefulHexRoll("p1", "t1", true, 4, false);
	}

	@Test
	public void serializationRoundTrip() {
		ReportBalefulHexRoll original = make();
		JsonObject json = (JsonObject) original.toJsonValue();
		ReportBalefulHexRoll restored = (ReportBalefulHexRoll) new ReportBalefulHexRoll().initFrom(ReportTestUtil.source(), json);
		assertEquals(original.getPlayerId(), restored.getPlayerId());
		assertEquals(original.isSuccessful(), restored.isSuccessful());
		assertEquals(original.getRoll(), restored.getRoll());
		assertEquals(original.getMinimumRoll(), restored.getMinimumRoll());
		assertEquals(original.isReRolled(), restored.isReRolled());
		assertEquals(original.getTarget(), restored.getTarget());
	}

	@Test
	public void toJsonValueHasReportId() {
		JsonObject json = (JsonObject) make().toJsonValue();
		assertEquals("balefulHex", json.get("reportId").asString());
	}
}
