package com.fumbbl.ffb.report;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.ReRollSources;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

public class ReportReRollTest {

	private ReportReRoll make() {
		return new ReportReRoll("p1", ReRollSources.TEAM_RE_ROLL, true, 4);
	}

	@Test
	public void serializationRoundTrip() {
		ReportReRoll original = make();
		JsonObject json = original.toJsonValue();
		ReportReRoll restored = new ReportReRoll().initFrom(ReportTestUtil.source(), json);
		assertEquals(original.getPlayerId(), restored.getPlayerId());
		assertEquals(original.getReRollSource().getName(), restored.getReRollSource().getName());
		assertEquals(original.isSuccessful(), restored.isSuccessful());
		assertEquals(original.getRoll(), restored.getRoll());
	}

	@Test
	public void toJsonValueHasReportId() {
		JsonObject json = make().toJsonValue();
		assertEquals("reRoll", json.get("reportId").asString());
	}
}
