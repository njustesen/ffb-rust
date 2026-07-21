package com.fumbbl.ffb.report.bb2016;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.report.ReportTestUtil;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertArrayEquals;
import static org.junit.jupiter.api.Assertions.assertEquals;

public class ReportApothecaryRollTest {

	private ReportApothecaryRoll make() {
		return new ReportApothecaryRoll("p1", new int[]{3, 4}, null, null);
	}

	@Test
	public void serializationRoundTrip() {
		ReportApothecaryRoll original = make();
		JsonObject json = original.toJsonValue();
		ReportApothecaryRoll restored = new ReportApothecaryRoll().initFrom(ReportTestUtil.source(), json);
		assertEquals(original.getPlayerId(), restored.getPlayerId());
		assertArrayEquals(original.getCasualtyRoll(), restored.getCasualtyRoll());
		assertEquals(original.getSeriousInjury(), restored.getSeriousInjury());
	}

	@Test
	public void toJsonValueHasReportId() {
		JsonObject json = make().toJsonValue();
		assertEquals("apothecaryRoll", json.get("reportId").asString());
	}
}
