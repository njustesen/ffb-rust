package com.fumbbl.ffb.report.mixed;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.report.ReportTestUtil;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

public class ReportThrownKegTest {

	private ReportThrownKeg make() {
		return new ReportThrownKeg("p1", "t1", 3, true, false);
	}

	@Test
	public void serializationRoundTrip() {
		ReportThrownKeg original = make();
		JsonObject json = original.toJsonValue().asObject();
		ReportThrownKeg restored = new ReportThrownKeg().initFrom(ReportTestUtil.source(), json);
		assertEquals(original.getPlayerId(), restored.getPlayerId());
		assertEquals(original.getTargetPlayerId(), restored.getTargetPlayerId());
		assertEquals(original.getRoll(), restored.getRoll());
		assertEquals(original.isSuccess(), restored.isSuccess());
		assertEquals(original.isFumble(), restored.isFumble());
	}

	@Test
	public void toJsonValueHasReportId() {
		JsonObject json = make().toJsonValue().asObject();
		assertEquals("thrownKeg", json.get("reportId").asString());
	}
}
