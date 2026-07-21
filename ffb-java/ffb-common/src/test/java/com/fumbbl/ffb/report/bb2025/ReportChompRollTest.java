package com.fumbbl.ffb.report.bb2025;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.report.ReportTestUtil;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

public class ReportChompRollTest {
	private ReportChompRoll make() {
		return new ReportChompRoll("p1", true, 5, 3, false, "chomper1", "chompee1");
	}

	@Test
	public void serializationRoundTrip() {
		ReportChompRoll original = make();
		JsonObject json = original.toJsonValue();
		ReportChompRoll restored = new ReportChompRoll().initFrom(ReportTestUtil.source(), json);
		assertEquals(original.getPlayerId(), restored.getPlayerId());
		assertEquals(original.isSuccessful(), restored.isSuccessful());
		assertEquals(original.getRoll(), restored.getRoll());
		assertEquals(original.getMinimumRoll(), restored.getMinimumRoll());
		assertEquals(original.isReRolled(), restored.isReRolled());
		assertEquals(original.getChomper(), restored.getChomper());
		assertEquals(original.getChompee(), restored.getChompee());
	}

	@Test
	public void toJsonValueHasReportId() {
		JsonObject json = make().toJsonValue();
		assertEquals("chompRoll", json.get("reportId").asString());
	}
}
